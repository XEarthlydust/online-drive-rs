use crate::module::error::AppError;
use chrono::{DateTime, Utc};
use rbatis::{impl_insert, impl_select, RBatis};
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct File {
    pub id: Option<Uuid>,
    pub create_time: Option<DateTime<Utc>>,
    pub delete_flag: Option<i8>,
    pub sha_256: Option<String>,
    pub path: Option<String>,
    pub file_type: Option<String>,
    pub size: Option<i64>,
    pub thumbnail: Option<String>,
}

impl File {
    pub fn uploaded_new(
        id: Uuid,
        sha_256: String,
        path: String,
        file_type: String,
        size: i64,
    ) -> File {
        File {
            id: Some(id),
            create_time: None,
            delete_flag: Some(0),
            sha_256: Some(sha_256),
            path: Some(path),
            file_type: Some(file_type),
            size: Some(size),
            thumbnail: None,
        }
    }

    pub fn new() -> File {
        File {
            id: Some(Uuid::new_v4()),
            create_time: None,
            delete_flag: Some(0),
            sha_256: None,
            path: Some(Uuid::new_v4().to_string()),
            file_type: None,
            size: None,
            thumbnail: None,
        }
    }
}

impl File {
    pub async fn update_thumbnail(
        rb: &RBatis,
        id: &Uuid,
        thumbnail_path: &String,
    ) -> Result<(), AppError> {
        let result: u64 = rb
            .exec(
                "UPDATE \"file\" SET thumbnail = ? WHERE id = ? AND delete_flag = 0",
                vec![rbs::to_value!(thumbnail_path), rbs::to_value!(id)],
            )
            .await?
            .rows_affected;
        (result == 0)
            .then(|| Err::<(), AppError>(AppError::FileNotExists))
            .transpose()?;
        Ok(())
    }

    pub async fn update_size_sha_256(
        rb: &RBatis,
        id: &Uuid,
        size: i64,
        hash: &String,
    ) -> Result<(), AppError> {
        let result: u64 = rb
            .exec(
                "UPDATE \"file\" SET size = ?, sha_256 = ? WHERE id = ? AND delete_flag = 0",
                vec![
                    rbs::to_value!(size),
                    rbs::to_value!(hash),
                    rbs::to_value!(id),
                ],
            )
            .await?
            .rows_affected;
        (result == 0)
            .then(|| Err::<(), AppError>(AppError::FileNotExists))
            .transpose()?;
        Ok(())
    }

    pub async fn delete_by_id(rb: &RBatis, id: &Uuid) -> Result<(), AppError> {
        rb.exec(
            "update \"file\" set delete_flag = 1 where id = ?",
            vec![rbs::to_value!(id)],
        )
        .await?;
        Ok(())
    }
}

impl_insert!(File {}, "\"file\"");
impl_select!(File {select_by_id(id: &Uuid) => "`where id = #{id} and delete_flag = 0 limit 1`"}, "\"file\"");
impl_select!(File {select_by_hash(sha_256: &String) => "`where sha_256 = #{sha_256} and delete_flag = 0 limit 1`"}, "\"file\"");
