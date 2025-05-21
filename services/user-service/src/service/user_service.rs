use argon2::{PasswordHash, PasswordVerifier};
use common::module::error::AppError;
use common::module::user::{User, UserVo};
use common::util::jwt;
use common::{argon2_client, config, db_pool};
use rbatis::{Page, PageRequest};
use uuid::Uuid;

pub struct UserService {}

impl UserService {
    pub async fn login(user_account: &String, password: &String) -> Result<String, AppError> {
        let hash_word_user = User::select_password_by_account(db_pool!(), user_account).await?;
        let hash_word = hash_word_user
            .user_password
            .ok_or(AppError::UserPasswordMismatch)?;
        let parsed_hash = PasswordHash::new(hash_word.as_str())?;
        match argon2_client!()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            false => Err(AppError::UserPasswordMismatch),
            true => {
                let user_id = hash_word_user
                    .id
                    .ok_or(AppError::InnerError("login-jwt-user_id".to_string()))?;
                let user_role = hash_word_user
                    .user_role
                    .ok_or(AppError::InnerError("login-jwt-user_role".to_string()))?;
                Ok(jwt::create_jwt(user_id, user_role, config!().jwt.exp_min)?)
            }
        }
    }
    pub async fn logout() -> Result<(), AppError> {
        Ok(())
    }
    pub async fn register(
        user_account: String,
        username: String,
        password: String,
        email: Option<String>,
        telephone: Option<String>,
    ) -> Result<u64, AppError> {
        User::exists_by_account(db_pool!(), &user_account)
            .await?
            .then(|| Err::<u64, AppError>(AppError::UserExists))
            .transpose()?;

        let new_user = User::new_as_register(user_account, username, email, telephone)
            .set_password(argon2_client!(), password)?;

        let row = User::insert(db_pool!(), &new_user).await?.rows_affected;
        Ok(row)
    }
    pub async fn change_userinfo(
        id: &Uuid,
        user_email: &String,
        sign: &String,
        telephone: &String,
    ) -> Result<(), AppError> {
        User::update_info_by_id(db_pool!(), id, user_email, sign, telephone).await?;
        Ok(())
    }
    pub async fn change_password(
        id: &Uuid,
        old_password: &String,
        new_password: &String,
    ) -> Result<(), AppError> {
        let hash_word_user = User::select_password_by_id(db_pool!(), id).await?;
        let hash_word = hash_word_user
            .user_password
            .ok_or(AppError::UserPasswordMismatch)?;
        let parsed_hash = PasswordHash::new(hash_word.as_str())?;
        match argon2_client!()
            .verify_password(old_password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            true => {
                User::update_password_by_id(
                    db_pool!(),
                    id,
                    &User::password_hash(argon2_client!(), new_password)?,
                )
                .await?;
                Ok(())
            }
            false => Err(AppError::UserPasswordMismatch),
        }
    }
    pub async fn delete(id: &Uuid) -> Result<(), AppError> {
        User::delete_by_id(db_pool!(), id).await?;
        Ok(())
    }
    pub async fn get_userinfo(user_id: &Uuid) -> Result<User, AppError> {
        let mut user = User::select_by_id(db_pool!(), user_id)
            .await?
            .into_iter()
            .next()
            .ok_or(AppError::UserNotExists)?;
        user.user_password = None;
        Ok(user)
    }

    pub async fn get_userinfo_public(
        user_id: &Uuid,
    ) -> Result<(Option<String>, Option<String>, Option<String>), AppError> {
        let mut user = User::select_by_id(db_pool!(), user_id)
            .await?
            .into_iter()
            .next()
            .ok_or(AppError::UserNotExists)?;
        user.user_password = None;
        Ok((user.username, user.avatar, user.sign))
    }

    pub async fn set_avatar(user_id: &Uuid, avatar_path: &String) -> Result<(), AppError> {
        User::update_avatar_by_id(db_pool!(), user_id, avatar_path).await?;
        Ok(())
    }

    pub async fn get_page_by_name(
        username: &String,
        page_no: u64,
        page_size: u64,
    ) -> Result<Page<UserVo>, AppError> {
        let data = UserVo::select_by_name_page(
            db_pool!(),
            &PageRequest::new(page_no, page_size),
            username,
        )
        .await?;
        Ok(data)
    }

    pub async fn set_user_max_size(user_id: &Uuid, max_size: &i64) -> Result<(), AppError> {
        User::update_max_size_by_id(db_pool!(), user_id, max_size).await?;
        Ok(())
    }
}
