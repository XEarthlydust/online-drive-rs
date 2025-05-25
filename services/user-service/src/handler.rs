use crate::service::user_service::UserService;
use aws_sdk_s3::primitives::ByteStream;
use common::module::error::AppError;
use common::module::user::{User, UserVo};
use common::util::jwt::Claims;
use common::util::result::{ResultCode, ResultData};
use common::{config, minio_client};
use rbatis::Page;
use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::oapi::ToSchema;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct PubicInfoVo {
    username: Option<String>,
    avatar: Option<String>,
    sign: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct LoginDto {
    user_account: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct RegisterDto {
    user_account: String,
    username: String,
    password: String,
    user_email: Option<String>,
    telephone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ChangePasswordDto {
    new_password: String,
    old_password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ChangeInfoDto {
    username: String,
    user_email: String,
    sign: String,
    telephone: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct UsersDto {
    username: String,
    page: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SizeDto {
    user_id: String,
    max_size: u64,
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Successfully login", body = ResultData<String>),
    )
)]
pub async fn login(
    login_dto: JsonBody<LoginDto>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let login_vo = login_dto.into_inner();
    let token = UserService::login(&login_vo.user_account, &login_vo.password).await?;
    res.render(Json(ResultData::new(
        "login success",
        Some(token),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Successfully register", body = ResultData<String>),
    )
)]
pub async fn register(
    register_dto: JsonBody<RegisterDto>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let register_vo = register_dto.into_inner();
    let username = register_vo.username.clone();
    UserService::register(
        register_vo.user_account,
        register_vo.username,
        register_vo.password,
        register_vo.user_email,
        register_vo.telephone,
    )
    .await?;
    res.render(Json(ResultData::<String>::new(
        username + " register success",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Successfully get userinfo", body = ResultData<User>),
    )
)]
pub async fn user_info(res: &mut Response, depot: &mut Depot) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    let user = UserService::get_userinfo(&claims.uid).await?;
    res.render(Json(ResultData::<User>::new(
        "get userinfo success",
        Some(user),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    parameters(
        ("user_id" = String, Path, description = "User id")
    ),
    responses(
        (status_code = 200, description = "Successfully get userinfo", body = ResultData<PubicInfoVo>),
    )
)]
pub async fn user_info_pubic(
    user_id: PathParam<Uuid>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let pubic_info_vo = {
        let (username, avatar, sign) =
            UserService::get_userinfo_public(&user_id.into_inner()).await?;
        PubicInfoVo {
            username,
            avatar,
            sign,
        }
    };

    res.render(Json(ResultData::<PubicInfoVo>::new(
        "get userinfo success",
        Some(pubic_info_vo),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Successfully update password", body = ResultData<String>),
    )
)]
pub async fn change_password(
    change_password_dto: JsonBody<ChangePasswordDto>,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    UserService::change_password(
        &claims.uid,
        &change_password_dto.old_password,
        &change_password_dto.new_password,
    )
    .await?;
    res.render(Json(ResultData::<String>::new(
        "change password success",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Token is valid", body = ResultData<String>),
    )
)]
pub async fn check_token(res: &mut Response, depot: &mut Depot) -> Result<StatusCode, AppError> {
    depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    res.render(Json(ResultData::<String>::new(
        "token is valid",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Token is valid", body = ResultData<String>),
    )
)]
pub async fn update_userinfo(
    change_info_dto: JsonBody<ChangeInfoDto>,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    UserService::change_userinfo(
        &claims.uid,
        &change_info_dto.username,
        &change_info_dto.user_email,
        &change_info_dto.sign,
        &change_info_dto.telephone,
    )
    .await?;
    res.render(Json(ResultData::<String>::new(
        "token is valid",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    parameters(
        ("user_id" = String, Path, description = "User id")
    ),
    responses(
        (status_code = 200, description = "Token is valid", body = ResultData<String>),
    )
)]
pub async fn delete_user(
    user_id: PathParam<Uuid>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    UserService::delete(&user_id.into_inner()).await?;

    res.render(Json(ResultData::<String>::new(
        "token is valid",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Logout", body = ResultData<String>),
    )
)]
pub async fn logout(res: &mut Response) -> Result<StatusCode, AppError> {
    UserService::logout().await?;
    res.render(Json(ResultData::<String>::new(
        "logout success",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Avatar", body = ResultData<String>),
    )
)]
pub async fn set_avatar(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    let max_size = 3 * 1024 * 1024;
    let key = Uuid::new_v4().to_string();
    let _ = match req.file("avatar").await {
        Some(file) => {
            if file.size() > max_size {
                return Err(AppError::MissingField("File too large".into()));
            }
            let path = ByteStream::from_path(file.path()).await;
            let path2 = path.map_err(|e| AppError::InnerError(e.to_string()))?;

            minio_client!()
                .put_object()
                .bucket("avatar")
                .key(key.clone())
                .body(path2)
                .send()
                .await?;

            UserService::set_avatar(&claims.uid, &key).await?;
        }
        None => return Err(AppError::MissingField("Missing multipart data".into())),
    };
    res.render(Json(ResultData::<String>::new(
        "Set avatar success",
        Some(key),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Get Users", body = ResultData<Vec<UserVo>>),
    )
)]
pub async fn get_users(
    users_dto: JsonBody<UsersDto>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let username = format!("%{}%", users_dto.username);
    let users =
        UserService::get_page_by_name(&username, users_dto.page, config!().page.size).await?;

    res.render(Json(ResultData::<Page<UserVo>>::new(
        "Get success",
        Some(users),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Set max size", body = ResultData<String>),
    )
)]
pub async fn set_user_max_size(
    size_dto: JsonBody<SizeDto>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::parse_str(size_dto.user_id.as_str()).map_err(|_e| AppError::UserNotExists)?;
    UserService::set_user_max_size(&uuid, &(size_dto.max_size as i64)).await?;
    res.render(Json(ResultData::<Page<UserVo>>::new(
        "Set success",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}
