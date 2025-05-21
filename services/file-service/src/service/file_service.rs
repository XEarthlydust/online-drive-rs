use aws_sdk_s3::types::CompletedPart;
use common::module::error::AppError;
use common::module::file::File;
use common::module::item::Item;
use common::util::hash::get_size_and_hash;
use common::util::minio::{
    complete_upload, generate_download_url, generate_part_upload_url, generate_upload_id,
};
use common::{config, db_pool, minio_client, req_client};
use rbatis::{Page, PageRequest};
use tracing::{error, info};
use uuid::Uuid;

pub struct FileService {}

impl FileService {
    // pub async fn check_parent_id(
    //     path: &FilePathInfo,
    //     user_id: Uuid,
    // ) -> Result<Option<Uuid>, AppError> {
    //     // let path = FilePathInfo::from_str(path)?;
    //     // Path check
    //     let parent_id = if path.parent != "/" {
    //         let p_items =
    //             Item::select_path_by_logic_name(db_pool!(), &path.parent, &user_id).await?;
    //         let p_item = p_items.get(0).ok_or(AppError::PathOrNameError)?;
    //         Some(
    //             p_item
    //                 .id
    //                 .ok_or_else(|| AppError::InnerError("upload-upload-get_item_id".into()))?,
    //         )
    //     } else {
    //         None
    //     };
    //     Ok(parent_id)
    // }
    pub async fn check_upload(
        user_id: Uuid,
        sha_256: &String,
        logic_name: String,
        parent_id: Option<Uuid>,
    ) -> Result<bool, AppError> {
        let files = File::select_by_hash(db_pool!(), sha_256).await?;
        if let Some(file) = files.get(0) {
            // Hash check, file find
            let file_id = file
                .id
                .ok_or_else(|| AppError::InnerError("upload-get_file_id".into()))?;
            let item = Item::new(user_id, Some(file_id), parent_id, false, logic_name, true);
            Item::insert(db_pool!(), &item).await?;
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    pub async fn start_upload(file: File) -> Result<String, AppError> {
        let minio_path = file.path.ok_or(AppError::FileNotExists)?;
        generate_upload_id(
            minio_client!(),
            config!().minio.file_bucket.as_str(),
            minio_path.as_str(),
        )
        .await
    }

    pub async fn get_part_upload_url(
        part: i64,
        upload_id: String,
        path: String,
    ) -> Result<String, AppError> {
        generate_part_upload_url(
            minio_client!(),
            config!().minio.file_bucket.as_str(),
            path.as_str(),
            part,
            upload_id,
        )
        .await
    }

    pub async fn set_completed_upload(
        user_id: Uuid,
        file_id: Option<Uuid>,
        parent_id: Option<Uuid>,
        logic_name: String,
        server_path: String,
        upload_id: &String,
        parts: Vec<CompletedPart>,
    ) -> Result<(), AppError> {
        complete_upload(
            minio_client!(),
            config!().minio.file_bucket.as_str(),
            format!("/{}", server_path.as_str()).as_str(),
            upload_id,
            parts,
        )
        .await?;
        tokio::spawn(async move {
            match Self::after_upload(user_id, file_id, parent_id, logic_name, &server_path).await {
                Ok(item_id) => {
                    info!(
                        "{} upload finish, user: {}, item: {}",
                        server_path, user_id, item_id
                    )
                }
                Err(e) => {
                    error!("{} upload fail, user: {}, E: {}", server_path, user_id, e)
                }
            }
        });
        Ok(())
    }

    pub async fn mkdir_item(
        user_id: Uuid,
        logic_name: String,
        parent_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let item = Item::new(user_id, None, parent_id, true, logic_name, true);
        Item::insert(db_pool!(), &item).await?;
        Ok(())
    }

    pub async fn rename_item(
        user_id: &Uuid,
        item_id: &Uuid,
        logic_name: &String,
    ) -> Result<File, AppError> {
        let file = File::new();
        Item::update_logic_name_by_id(db_pool!(), item_id, user_id, logic_name).await?;
        Ok(file)
    }

    pub async fn get_download_url(file_id: &Uuid, file_name: &String) -> Result<String, AppError> {
        let file_path = File::select_by_id(db_pool!(), file_id)
            .await?
            .get(0)
            .ok_or(AppError::FileNotExists)?
            .path
            .clone()
            .ok_or(AppError::FileNotExists)?;
        let url = generate_download_url(
            minio_client!(),
            config!().minio.file_bucket.as_str(),
            file_path.as_str(),
            file_name,
        )
        .await?;
        Ok(url)
    }

    pub async fn get_item_list(
        user_id: Uuid,
        parent_id: Option<Uuid>,
        page_no: u64,
        page_size: u64,
    ) -> Result<Page<Item>, AppError> {
        let data = match parent_id {
            Some(parent_id) => {
                Item::select_page_by_parent(
                    db_pool!(),
                    &PageRequest::new(page_no, page_size),
                    &parent_id,
                    &user_id,
                )
                .await?
            }
            None => {
                Item::select_page_root(db_pool!(), &PageRequest::new(page_no, page_size), &user_id)
                    .await?
            }
        };
        Ok(data)
    }

    pub async fn delete_item(user_id: &Uuid, item_id: &Uuid) -> Result<(), AppError> {
        let item_vec = Item::select_by_id_userid(db_pool!(), item_id, user_id).await?;
        let item = item_vec.get(0).ok_or(AppError::FileNotExists)?;
        if item.is_folder.ok_or(AppError::FileNotExists)? {
            Item::delete_sub_by_id(db_pool!(), item_id, user_id).await?;
            Item::delete_by_id(db_pool!(), item_id, user_id).await?;
        }
        Item::delete_by_id(db_pool!(), item_id, user_id).await?;
        Ok(())
    }

    pub async fn get_item_to_file_id(user_id: &Uuid, item_id: &Uuid) -> Result<Uuid, AppError> {
        let item_vec = Item::select_by_id_userid(db_pool!(), &item_id, &user_id).await?;
        let item = item_vec.get(0).ok_or(AppError::FileNotExists)?;
        item.file_id.ok_or(AppError::FileNotExists)
    }

    pub async fn get_item_by_id(user_id: &Uuid, item_id: &Uuid) -> Result<Item, AppError> {
        let item_vec = Item::select_by_id_userid(db_pool!(), &user_id, &item_id).await?;
        let item = item_vec.get(0).ok_or(AppError::FileNotExists)?.clone();
        Ok(item)
    }

    pub async fn create_new_file() -> Result<File, AppError> {
        let file = File::new();
        File::insert(db_pool!(), &file).await?;
        Ok(file)
    }

    pub async fn move_item(
        user_id: &Uuid,
        item_id: &Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<File, AppError> {
        let file = File::new();
        Item::update_parent_by_id(db_pool!(), item_id, user_id, parent_id).await?;
        Ok(file)
    }

    pub async fn after_upload(
        user_id: Uuid,
        file_id: Option<Uuid>,
        parent_id: Option<Uuid>,
        logic_name: String,
        server_path: &String,
    ) -> Result<Uuid, AppError> {
        let item = Item::new(user_id, file_id, parent_id, false, logic_name, true);
        Item::insert(db_pool!(), &item).await?;

        // need to put in other thread
        let download_url = generate_download_url(
            minio_client!(),
            config!().minio.file_bucket.as_str(),
            server_path.as_str(),
            file_id
                .ok_or(AppError::InnerError("file-id".into()))?
                .to_string()
                .as_str(),
        )
        .await?;
        let (size, hash) = get_size_and_hash(req_client!(), download_url.as_str()).await?;
        File::update_size_sha_256(
            db_pool!(),
            &file_id.ok_or(AppError::InnerError("get-file-id error".into()))?,
            size,
            &hash,
        )
        .await?;
        Ok(item.id.ok_or(AppError::FileNotExists)?)
    }
}
