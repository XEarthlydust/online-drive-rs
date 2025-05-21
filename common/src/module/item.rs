use crate::module::error::AppError;
use chrono::{DateTime, Utc};
use rbatis::{impl_insert, impl_select, impl_select_page, RBatis};
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: Option<Uuid>,
    pub create_time: Option<DateTime<Utc>>,
    pub delete_flag: Option<i8>,
    pub user_id: Option<Uuid>,
    pub file_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub is_folder: Option<bool>,
    pub logic_name: Option<String>,
    pub uploaded: Option<bool>,
}

impl Item {
    pub fn new(
        user_id: Uuid,
        file_id: Option<Uuid>,
        parent_id: Option<Uuid>,
        is_folder: bool,
        logic_name: String,
        uploaded: bool,
    ) -> Item {
        Item {
            id: Some(Uuid::new_v4()),
            create_time: None,
            delete_flag: Some(0),
            user_id: Some(user_id),
            file_id,
            parent_id,
            is_folder: Some(is_folder),
            logic_name: Some(logic_name),
            uploaded: Some(uploaded),
        }
    }
    pub fn set_uploaded(&self, uploaded: bool) -> Item {
        let mut new_self = self.clone();
        new_self.uploaded = Some(uploaded);
        new_self
    }
}

impl Item {
    pub async fn update_uploaded_by_id(rb: &RBatis, id: &Uuid) -> Result<(), AppError> {
        let result: u64 = rb
            .exec(
                "UPDATE \"item\" SET uploaded = true WHERE id = ? AND delete_flag = 0",
                vec![rbs::to_value!(id)],
            )
            .await?
            .rows_affected;
        (result == 0)
            .then(|| Err::<(), AppError>(AppError::ItemNotExists))
            .transpose()?;
        Ok(())
    }
    pub async fn delete_by_id(rb: &RBatis, id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        rb.exec(
            "update \"item\" set delete_flag = 1 where id = ? and user_id = ?",
            vec![rbs::to_value!(id), rbs::to_value!(user_id)],
        )
        .await?;
        Ok(())
    }

    pub async fn update_parent_by_id(
        rb: &RBatis,
        id: &Uuid,
        user_id: &Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        rb.exec(
            "update \"item\" set parent_id = ? where id = ? and user_id = ? and delete_flag = 0",
            vec![
                rbs::to_value!(parent_id),
                rbs::to_value!(id),
                rbs::to_value!(user_id),
            ],
        )
        .await?;
        Ok(())
    }

    pub async fn delete_sub_by_id(rb: &RBatis, id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        rb.exec(
            "update \"item\" set delete_flag = 1 where parent_id = ? and user_id = ?",
            vec![rbs::to_value!(id), rbs::to_value!(user_id)],
        )
        .await?;
        Ok(())
    }

    pub async fn update_logic_name_by_id(
        rb: &RBatis,
        id: &Uuid,
        user_id: &Uuid,
        logic_name: &String,
    ) -> Result<(), AppError> {
        rb.exec(
            "update \"item\" set logic_name = ? where id = ? and user_id = ?",
            vec![
                rbs::to_value!(logic_name),
                rbs::to_value!(id),
                rbs::to_value!(user_id),
            ],
        )
        .await?;
        Ok(())
    }
}

impl_insert!(Item {}, "\"item\"");
impl_select!(Item {select_by_id(id: &Uuid) => "`where id = #{id} and delete_flag = 0 limit 1`"}, "\"item\"");
impl_select!(Item {select_by_id_userid(id: &Uuid, user_id: &Uuid) => "`where id = #{id} and user_id = #{user_id} and delete_flag = 0 limit 1`"}, "\"item\"");
impl_select!(Item {select_path_by_logic_name(logic_name: &String, user_id: &Uuid) => "`where logic_name = #{logic_name} and user_id = #{user_id} and delete_flag = 0 and is_folder = true limit 1`"}, "\"item\"");
impl_select_page!(Item {select_page_by_parent(id:&Uuid, user_id: &Uuid) => "where parent_id = #{id} and user_id = #{user_id} and delete_flag = 0"}, "\"item\"");
impl_select_page!(Item {select_page_root(user_id: &Uuid) => "where parent_id is null and user_id = #{user_id} and delete_flag = 0"}, "\"item\"");
