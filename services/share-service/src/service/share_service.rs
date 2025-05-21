use chrono::{DateTime, Utc};
use common::db_pool;
use common::module::error::AppError;
use common::module::item::Item;
use common::module::share::Share;
use common::util::path::FilePathInfo;
use uuid::Uuid;

pub struct ShareService {}

impl ShareService {
    pub async fn try_get_share(share_id: &Uuid) -> Result<Share, AppError> {
        let share = Share::select_by_id(db_pool!(), share_id)
            .await?
            .first()
            .ok_or(AppError::ShareFileNotFound)?
            .to_owned();
        Ok(share)
    }

    pub async fn get_share(share_id: &Uuid, pickup_code: &String) -> Result<Share, AppError> {
        let share = Share::select_by_id(db_pool!(), share_id)
            .await?
            .first()
            .ok_or(AppError::ShareFileNotFound)?
            .to_owned();

        match share.is_public {
            Some(true) => Ok(share),
            Some(false) => match &share.pickup_code {
                Some(code) if code == pickup_code => Ok(share),
                _ => Err(AppError::ShareCodeMismatched),
            },
            None => Err(AppError::ShareFileNotFound),
        }
    }

    pub async fn get_share_name(share_id: &Uuid) -> Result<String, AppError> {
        Share::get_logic_name_by_id(db_pool!(), share_id).await
    }

    pub async fn create_share(
        user_id: Uuid,
        item_id: Uuid,
        is_public: bool,
        pickup_code: Option<String>,
        timeout_time: Option<DateTime<Utc>>,
    ) -> Result<Share, AppError> {
        let share = Share::new(
            Some(user_id),
            Some(item_id),
            Some(is_public),
            pickup_code,
            timeout_time,
        );
        Share::insert(db_pool!(), &share).await?;
        Ok(share)
    }

    pub async fn delete_share(user_id: &Uuid, share_id: &Uuid) -> Result<(), AppError> {
        Share::delete_by_id(db_pool!(), share_id, user_id).await?;
        Ok(())
    }

    pub async fn timeout_delete_share(share_id: &Uuid) -> Result<(), AppError> {
        Share::timeout_delete_by_id(db_pool!(), share_id).await?;
        Ok(())
    }

    pub async fn get_user_shares(user_id: &Uuid) -> Result<Vec<Share>, AppError> {
        let shares = Share::select_page_by_userid(db_pool!(), user_id).await?;
        Ok(shares)
    }

    pub async fn save_share_file(
        user_id: Uuid,
        parent_id: Option<Uuid>,
        logic_name: String,
        item_id: &Uuid,
        share_id: &Uuid,
    ) -> Result<(), AppError> {
        let item = Item::select_by_id(db_pool!(), item_id)
            .await?
            .first()
            .ok_or(AppError::ItemNotExists)?
            .to_owned();
        let item = Item::new(user_id, item.file_id, parent_id, false, logic_name, true);
        Item::insert(db_pool!(), &item).await?;
        Share::add_once_save_times_by_id(db_pool!(), share_id).await?;
        Ok(())
    }

    pub async fn check_parent_id(
        path: &FilePathInfo,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let parent_id = if path.parent != "/" {
            let p_items =
                Item::select_path_by_logic_name(db_pool!(), &path.parent, &user_id).await?;
            let p_item = p_items.first().ok_or(AppError::PathOrNameError)?;
            Some(
                p_item
                    .id
                    .ok_or_else(|| AppError::InnerError("save-share-get_item_id".into()))?,
            )
        } else {
            None
        };
        Ok(parent_id)
    }
}
