use crate::service::share_service::ShareService;
use chrono::{Duration, Utc};
use common::module::error::AppError;
use common::module::share::{Share, ShareVo, ShareVoWithName};
use common::util::jwt::Claims;
use common::util::result::{ResultCode, ResultData};
use salvo::oapi::extract::{JsonBody, QueryParam};
use salvo::oapi::ToSchema;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct GetShareDto {
    share_id: Uuid,
    code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ShareItemDto {
    days: u32,
    item_id: Uuid,
    is_public: bool,
    code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SaveShareDto {
    share_id: Uuid,
    code: Option<String>,
    logic_name: String,
    parent_id: Option<Uuid>,
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "get share without code", body = ResultData<bool>),
    )
)]
pub async fn get_share_publicly(
    get_share_dto: JsonBody<GetShareDto>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let share = ShareVo::from_share(ShareService::try_get_share(&get_share_dto.share_id).await?)?;

    if share.timeout_time.ok_or(AppError::ShareFileNotFound)? < Utc::now() {
        ShareService::timeout_delete_share(&share.id).await?;
        return Err(AppError::ShareFileNotFound);
    }

    let a = match share.is_public {
        Some(true) => Ok(true),
        Some(false) => Ok(false),
        None => Err(AppError::ShareFileNotFound),
    }?;

    res.render(Json(ResultData::<bool>::new(
        "Success",
        Some(a),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "get share with code", body = ResultData<ShareVoWithName>),
    )
)]
pub async fn get_share_with_code(
    get_share_dto: JsonBody<GetShareDto>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let code = get_share_dto
        .code
        .clone()
        .ok_or(AppError::MissingField("code".into()))?;
    let share = ShareVoWithName::from_share(
        ShareService::get_share(&get_share_dto.share_id, &code).await?,
    )?;

    if share.timeout_time.ok_or(AppError::ShareFileNotFound)? < Utc::now() {
        ShareService::timeout_delete_share(&share.id).await?;
        return Err(AppError::ShareFileNotFound);
    }
    let name = ShareService::get_share_name(&share.id).await?;
    let share = share.set_logic_name(name);
    res.render(Json(ResultData::<ShareVoWithName>::new(
        "Success",
        Some(share),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "get shares", body = ResultData<Vec<ShareVo>>),
    )
)]
pub async fn get_user_shares(
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    let shares = ShareService::get_user_shares(&claims.uid).await?;
    res.render(Json(ResultData::<Vec<Share>>::new(
        "Success",
        Some(shares),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "create share", body = ResultData<ShareVo>),
    )
)]
pub async fn create_share(
    share_item_dto: JsonBody<ShareItemDto>,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    let timeout_time = Utc::now() + Duration::days(share_item_dto.days as i64);
    println!("{:?}", timeout_time);
    let share = ShareService::create_share(
        claims.uid,
        share_item_dto.item_id,
        share_item_dto.is_public,
        share_item_dto.code.clone(),
        Some(timeout_time),
    )
    .await?;
    let share = ShareVo::from_share(share)?;
    res.render(Json(ResultData::<ShareVo>::new(
        "Success",
        Some(share),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    parameters(
        ("sid" = String, Path, description = "Share id")
    ),
    responses(
        (status_code = 200, description = "delete share", body = ResultData<String>),
    )
)]
pub async fn delete_share(
    sid: QueryParam<Uuid, true>,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    ShareService::delete_share(&claims.uid, &sid.into_inner()).await?;
    res.render(Json(ResultData::<ShareVo>::new(
        "Success",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "save share", body = ResultData<String>),
    )
)]
pub async fn save_share(
    save_share_dto: JsonBody<SaveShareDto>,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    let parent_id = save_share_dto.parent_id;
    let code = save_share_dto
        .code
        .clone()
        .ok_or(AppError::MissingField("code".into()))?;
    let share =
        ShareVo::from_share(ShareService::get_share(&save_share_dto.share_id, &code).await?)?;

    if share.timeout_time.ok_or(AppError::ShareFileNotFound)? < Utc::now() {
        ShareService::timeout_delete_share(&share.id).await?;
        return Err(AppError::ShareFileNotFound);
    }

    ShareService::save_share_file(
        claims.uid,
        parent_id,
        save_share_dto.logic_name.clone(),
        &share.item_id.ok_or(AppError::ShareFileNotFound)?,
        &share.id,
    )
    .await?;
    res.render(Json(ResultData::<ShareVo>::new(
        "Success",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}
