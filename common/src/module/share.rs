use crate::module::error::AppError;
use chrono::{DateTime, Utc};
use rbatis::{impl_select, RBatis};
use rbs::from_value;
use salvo::oapi::ToSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct Share {
    pub id: Option<Uuid>,
    pub create_time: Option<DateTime<Utc>>,
    pub timeout_time: Option<DateTime<Utc>>,
    pub delete_flag: Option<i8>,
    pub user_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub is_public: Option<bool>,
    pub pickup_code: Option<String>,
    pub save_times: Option<i64>,
}

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct ShareVo {
    pub id: Uuid,
    pub create_time: Option<DateTime<Utc>>,
    pub timeout_time: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub is_public: Option<bool>,
    pub save_times: Option<i64>,
}

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct ShareVoWithName {
    pub id: Uuid,
    pub create_time: Option<DateTime<Utc>>,
    pub timeout_time: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub is_public: Option<bool>,
    pub logic_name: Option<String>,
    pub save_times: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct LogicName {
    logic_name: String,
}

impl Share {
    pub fn new(
        user_id: Option<Uuid>,
        item_id: Option<Uuid>,
        is_public: Option<bool>,
        pickup_code: Option<String>,
        timeout_time: Option<DateTime<Utc>>,
    ) -> Self {
        Share {
            id: Some(Uuid::new_v4()),
            create_time: None,
            timeout_time,
            delete_flag: Some(0),
            user_id,
            item_id,
            is_public,
            pickup_code,
            save_times: Some(0),
        }
    }
    pub async fn delete_by_id(rb: &RBatis, id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        rb.exec(
            "update \"share\" set delete_flag = 1 where id = ? and user_id = ?",
            vec![rbs::to_value!(id), rbs::to_value!(user_id)],
        )
        .await?;
        Ok(())
    }

    pub async fn timeout_delete_by_id(rb: &RBatis, id: &Uuid) -> Result<(), AppError> {
        rb.exec(
            "update \"share\" set delete_flag = 1 where id = ?",
            vec![rbs::to_value!(id)],
        )
        .await?;
        Ok(())
    }

    pub async fn add_once_save_times_by_id(rb: &RBatis, id: &Uuid) -> Result<(), AppError> {
        rb.exec(
            "update \"share\" set save_times = save_times + 1 where id = ?",
            vec![rbs::to_value!(id)],
        )
        .await?;
        Ok(())
    }

    pub async fn insert(rb: &RBatis, share: &Share) -> Result<(), AppError> {
        rb.exec(
            "insert into share (id, user_id, item_id, is_public, timeout_time, pickup_code) values (?, ?, ?, ?, ?::timestamptz, ?)",
            vec![rbs::to_value!(share.id), rbs::to_value!(share.user_id), rbs::to_value!(share.item_id), rbs::to_value!(share.is_public), rbs::to_value!(share.timeout_time), rbs::to_value!(share.pickup_code.clone())],
        )
            .await?;
        Ok(())
    }

    pub async fn get_logic_name_by_id(rb: &RBatis, share_id: &Uuid) -> Result<String, AppError> {
        let (_, val) = rb
            .query(
                "SELECT i.logic_name FROM share s INNER JOIN item i ON s.item_id = i.id WHERE s.delete_flag = 0 AND s.id=? LIMIT 1;",
                vec![rbs::to_value!(share_id)],
            )
            .await?
            .into_iter().next().ok_or(AppError::ShareFileNotFound)?;
        let logic_name: LogicName = from_value(val).map_err(|_| AppError::ShareFileNotFound)?;
        Ok(logic_name.logic_name)
    }
}

impl_select!(Share {select_by_id(id: &Uuid) => "`where id = #{id} and delete_flag = 0 limit 1`"}, "\"share\"");
impl_select!(ShareVo {select_page_by_userid(id: &Uuid) => "`where user_id = #{id} and delete_flag = 0`"}, "\"share\"");
impl_select!(Share {select_page_by_userid(id: &Uuid) => "`where user_id = #{id} and delete_flag = 0`"}, "\"share\"");

impl ShareVo {
    pub fn from_share(share: Share) -> Result<Self, AppError> {
        Ok(ShareVo {
            id: share.id.ok_or(AppError::ShareFileNotFound)?,
            create_time: share.create_time,
            timeout_time: share.timeout_time,
            user_id: share.user_id,
            item_id: share.item_id,
            is_public: share.is_public,
            save_times: share.save_times,
        })
    }
}

impl ShareVoWithName {
    pub fn from_share(share: Share) -> Result<Self, AppError> {
        Ok(ShareVoWithName {
            id: share.id.ok_or(AppError::ShareFileNotFound)?,
            create_time: share.create_time,
            timeout_time: share.timeout_time,
            user_id: share.user_id,
            item_id: share.item_id,
            is_public: share.is_public,
            save_times: share.save_times,
            logic_name: None,
        })
    }

    pub fn set_logic_name(mut self, name: String) -> Self {
        self.logic_name = Some(name);
        self
    }
}
