use crate::module::error::AppError;
use chrono::{DateTime, Utc};
use rbatis::{impl_insert, impl_select_page, RBatis};
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct Commit {
    pub id: Option<Uuid>,
    pub create_time: Option<DateTime<Utc>>,
    pub delete_flag: Option<i8>,
    pub share_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub context: Option<String>,
}

impl Commit {
    pub fn new(share_id: Uuid, user_id: Uuid, context: String) -> Self {
        Commit {
            id: Some(Uuid::new_v4()),
            create_time: Some(Utc::now()),
            delete_flag: Some(0),
            share_id: Some(share_id),
            user_id: Some(user_id),
            context: Some(context),
        }
    }

    pub async fn delete_by_id(
        rb: &RBatis,
        commit_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), AppError> {
        rb.exec(
            "update \"commit\" set delete_flag = 1 where id = ? and user_id = ?",
            vec![rbs::to_value!(commit_id), rbs::to_value!(user_id)],
        )
        .await?;
        Ok(())
    }
}

impl_insert!(Commit {}, "\"commit\"");
impl_select_page!(Commit {select_page_by_shareid(id: &Uuid) => "`where share_id = #{id} and delete_flag = 0`"}, "\"commit\"");
