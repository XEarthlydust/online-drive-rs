use crate::module::error::AppError;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::{DateTime, Utc};
use rbatis::{impl_insert, impl_select, impl_select_page, RBatis};
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Option<Uuid>,
    pub username: Option<String>,
    pub user_account: Option<String>,
    pub user_password: Option<String>,
    pub user_role: Option<String>,
    pub user_email: Option<String>,
    pub sign: Option<String>,
    pub telephone: Option<String>,
    pub avatar: Option<String>,
    pub delete_flag: Option<i8>,
    pub create_time: Option<DateTime<Utc>>,
    pub max_size: Option<i64>,
    pub total_size: Option<i64>,
}

#[derive(ToSchema, Clone, Debug, Serialize, Deserialize)]
pub struct UserVo {
    pub id: Option<Uuid>,
    pub username: Option<String>,
    pub user_account: Option<String>,
    pub user_role: Option<String>,
    pub user_email: Option<String>,
    pub sign: Option<String>,
    pub telephone: Option<String>,
    pub avatar: Option<String>,
    pub delete_flag: Option<i8>,
    pub create_time: Option<DateTime<Utc>>,
    pub max_size: Option<i64>,
    pub total_size: Option<i64>,
}

impl User {
    pub fn set_password(
        mut self,
        argon2: &Argon2,
        password: String,
    ) -> Result<User, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        self.user_password = Some(password_hash.to_string());
        Ok(self)
    }

    pub fn new_as_register(
        user_account: String,
        username: String,
        email: Option<String>,
        telephone: Option<String>,
    ) -> User {
        User {
            id: Some(Uuid::new_v4()),
            user_account: Some(user_account),
            user_password: None,
            username: Some(username),
            user_email: email,
            user_role: Some("user".to_string()),
            sign: None,
            telephone,
            avatar: None,
            delete_flag: Some(0),
            max_size: Some(1073741824),
            total_size: Some(0),
            create_time: None,
        }
    }

    pub fn password_hash(argon2: &Argon2, password: &String) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    }
}

impl User {
    pub async fn exists_by_account(rb: &RBatis, account: &String) -> rbatis::Result<bool> {
        let val = rb
            .query(
                "select 1 from \"user\" where user_account = ? and delete_flag = 0 limit 1",
                vec![rbs::to_value!(account)],
            )
            .await?
            .into_iter()
            .next();
        Ok(val.is_some())
    }

    pub async fn select_password_by_account(
        rb: &RBatis,
        account: &String,
    ) -> Result<User, AppError> {
        let result: Option<User> = rb
            .query_decode(
                "SELECT id, user_role, user_password FROM \"user\" WHERE user_account = ? AND delete_flag = 0 LIMIT 1",
                vec![rbs::to_value!(account)],
            )
            .await?;
        match result {
            Some(u) => Ok(u),
            None => Err(AppError::UserPasswordMismatch),
        }
    }

    pub async fn select_password_by_id(rb: &RBatis, id: &Uuid) -> Result<User, AppError> {
        let result: Option<User> = rb
            .query_decode(
                "SELECT id, user_role, user_password FROM \"user\" WHERE id = ? AND delete_flag = 0 LIMIT 1",
                vec![rbs::to_value!(id)],
            )
            .await?;
        match result {
            Some(u) => Ok(u),
            None => Err(AppError::UserPasswordMismatch),
        }
    }

    pub async fn update_info_by_id(
        rb: &RBatis,
        id: &Uuid,
        user_email: &String,
        sign: &String,
        telephone: &String,
    ) -> Result<(), AppError> {
        let result: u64 = rb.exec(
            "UPDATE \"user\" SET user_email = ?, sign = ?, telephone = ? WHERE id = ? AND delete_flag = 0",
            vec![rbs::to_value!(user_email), rbs::to_value!(sign), rbs::to_value!(telephone), rbs::to_value!(id)]
        ).await?.rows_affected;
        (result == 0)
            .then(|| Err::<(), AppError>(AppError::UserNotExists))
            .transpose()?;
        Ok(())
    }

    pub async fn update_password_by_id(
        rb: &RBatis,
        id: &Uuid,
        user_password: &String,
    ) -> Result<(), AppError> {
        let result: u64 = rb
            .exec(
                "UPDATE \"user\" SET user_password = ? WHERE id = ? AND delete_flag = 0",
                vec![rbs::to_value!(user_password), rbs::to_value!(id)],
            )
            .await?
            .rows_affected;
        (result == 0)
            .then(|| Err::<(), AppError>(AppError::UserNotExists))
            .transpose()?;
        Ok(())
    }

    pub async fn update_avatar_by_id(
        rb: &RBatis,
        id: &Uuid,
        avatar: &String,
    ) -> Result<(), AppError> {
        let result: u64 = rb
            .exec(
                "UPDATE \"user\" SET avatar = ? WHERE id = ? AND delete_flag = 0",
                vec![rbs::to_value!(avatar), rbs::to_value!(id)],
            )
            .await?
            .rows_affected;
        (result == 0)
            .then(|| Err::<(), AppError>(AppError::UserNotExists))
            .transpose()?;
        Ok(())
    }

    pub async fn delete_by_id(rb: &RBatis, id: &Uuid) -> Result<(), AppError> {
        rb.exec(
            "update \"user\" set delete_flag = 1 where id = ?",
            vec![rbs::to_value!(id)],
        )
        .await?;
        Ok(())
    }

    pub async fn update_total_size_by_id(
        rb: &RBatis,
        id: &Uuid,
        size: &i64,
    ) -> Result<(), AppError> {
        rb.exec(
            "update \"user\" set total_size = total_size + ? where id = ? and delete_flag = 0",
            vec![rbs::to_value!(size), rbs::to_value!(id)],
        )
        .await?;
        Ok(())
    }

    pub async fn update_max_size_by_id(rb: &RBatis, id: &Uuid, size: &i64) -> Result<(), AppError> {
        rb.exec(
            "update \"user\" set max_size = ? where id = ? and delete_flag = 0",
            vec![rbs::to_value!(size), rbs::to_value!(id)],
        )
        .await?;
        Ok(())
    }
}

impl_insert!(User {}, "\"user\"");
impl_select!(User {select_by_id(id: &Uuid) => "`where id = #{id} and delete_flag = 0 limit 1`"}, "\"user\"");
impl_select_page!(UserVo {select_by_name_page(username: &String) => "`where username like #{username} and delete_flag = 0`"}, "\"user\"");
