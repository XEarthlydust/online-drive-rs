use crate::service::file_service::FileService;
use aws_sdk_s3::types::CompletedPart;
use common::{config, db_pool};
use common::module::error::AppError;
use common::module::item::Item;
use common::util::jwt::{create_payload, validate_payload, Claims, Operation};
use common::util::result::{ResultCode, ResultData};
use salvo::http::StatusCode;
use salvo::oapi::extract::{JsonBody, QueryParam};
use salvo::oapi::ToSchema;
use salvo::prelude::*;
use salvo::Response;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use common::module::user::User;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct HashUploadDto {
    hash: String,
    parent_id: Option<Uuid>,
    logic_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct MakeDirDto {
    parent_id: Option<Uuid>,
    logic_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct PartUploadDto {
    part: i64,
    credentials: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct FinishUploadedDto {
    logic_name: String,
    parent_id: Option<Uuid>,
    credentials: String,
    parts: Vec<Part>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
struct Part {
    part: i64,
    etag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
struct ItemsDto {
    page: u64,
    parent_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
struct MoveItemDto {
    item_id: Uuid,
    parent_id: Option<Uuid>,
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Check file in server", body = ResultData<bool>),
    )
)]
pub async fn upload_by_hash(
    hash_upload_dto: JsonBody<HashUploadDto>,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    let parent_id = hash_upload_dto.parent_id;
    let success = FileService::check_upload(
        claims.uid,
        &hash_upload_dto.hash,
        hash_upload_dto.logic_name.clone(),
        parent_id,
    )
    .await?;
    res.render(Json(ResultData::<bool>::new(
        "Check hash finished",
        Some(success),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}
#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Start upload", body = ResultData<String>),
    )
)]
pub async fn start_upload_file(res: &mut Response) -> Result<StatusCode, AppError> {
    // After hash check, file not find, start upload
    let file = FileService::create_new_file().await?;
    let file_id = file
        .id
        .ok_or(AppError::InnerError("Spawn file_id error".to_string()))?;
    let path = file.path.clone();
    let upload_id = FileService::start_upload(file).await?;
    let payload = create_payload(file_id, upload_id, path, Operation::FromFileStartUpload)?;
    res.render(Json(ResultData::<String>::new(
        "Can start upload",
        Some(payload),
        ResultCode::Success,
    )));
    Ok(StatusCode::CREATED)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Get upload url", body = ResultData<String>),
    )
)]
pub async fn upload_file_part(
    part_upload_dto: JsonBody<PartUploadDto>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let payload = validate_payload(part_upload_dto.credentials.as_str())?;
    if payload.operation != Operation::FromFileStartUpload {
        return Err(AppError::PayloadInvalid);
    }
    let upload_path = payload.data.clone();
    let url = FileService::get_part_upload_url(
        part_upload_dto.part,
        payload.id.clone(),
        upload_path.ok_or(AppError::PayloadInvalid)?,
    )
    .await?;
    res.render(Json(ResultData::<String>::new(
        format!("Get {} part url", part_upload_dto.part),
        Some(url),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(201),
    responses(
        (status_code = 201, description = "Finish upload", body = ResultData<String>),
    )
)]
pub async fn finish_upload(
    finish_uploaded_dto: JsonBody<FinishUploadedDto>,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;

    let payload = validate_payload(finish_uploaded_dto.credentials.as_str())?;
    if payload.operation != Operation::FromFileStartUpload {
        return Err(AppError::PayloadInvalid);
    }

    let completed_parts: Vec<CompletedPart> = finish_uploaded_dto
        .parts
        .clone()
        .into_iter()
        .map(|part| {
            CompletedPart::builder()
                .e_tag(part.etag)
                .part_number(part.part as i32)
                .build()
        })
        .collect();

    FileService::set_completed_upload(
        claims.uid,
        Some(payload.uid),
        finish_uploaded_dto.parent_id,
        finish_uploaded_dto.logic_name.clone(),
        payload
            .data
            .ok_or(AppError::InnerError("finish_upload error".to_string()))?,
        &payload.id,
        completed_parts,
    )
    .await?;

    res.render(Json(ResultData::<String>::new(
        "Completed upload",
        None,
        ResultCode::Success,
    )));

    Ok(StatusCode::CREATED)
}

#[endpoint(
    status_codes(201),
    responses(
        (status_code = 201, description = "Make dir", body = ResultData<String>),
    )
)]
pub async fn make_logic_dir(
    make_dir_dto: JsonBody<MakeDirDto>,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    let parent_id = make_dir_dto.parent_id;
    FileService::mkdir_item(claims.uid, make_dir_dto.logic_name.clone(), parent_id).await?;

    res.render(Json(ResultData::<String>::new(
        "Make dir",
        None,
        ResultCode::Success,
    )));

    Ok(StatusCode::CREATED)
}

#[endpoint(
    status_codes(200),
    parameters(
        ("iid" = String, Path, description = "File id")
    ),
    responses(
        (status_code = 200, description = "Get download url", body = ResultData<String>),
    )
)]
pub async fn download(
    iid: QueryParam<Uuid, true>,
    name: QueryParam<String, true>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let url = FileService::get_download_url(&iid, &name).await?;
    res.render(Json(ResultData::<String>::new(
        "Completed get download url",
        Some(url),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    parameters(
        ("iid" = String, Path, description = "File id")
    ),
    responses(
        (status_code = 200, description = "Delete item", body = ResultData<String>),
    )
)]
pub async fn delete(
    iid: QueryParam<Uuid, true>,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    FileService::delete_item(&claims.uid, &iid.into_inner()).await?;
    res.render(Json(ResultData::<String>::new(
        "Completed deleted",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "Get items", body = ResultData<Vec<Item>>),
    )
)]
pub async fn get_item(
    items_dto: JsonBody<ItemsDto>,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    let item_id = items_dto.parent_id;
    let items =
        FileService::get_item_list(claims.uid, item_id, items_dto.page, config!().page.size)
            .await?;

    res.render(Json(ResultData::<Vec<Item>>::new(
        "Get success",
        Some(items.records),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    parameters(
        ("iid" = String, Path, description = "File id"),
        ("name" = String, Path, description = "File id")
    ),
    responses(
        (status_code = 200, description = "Rename item", body = ResultData<String>),
    )
)]
pub async fn rename(
    iid: QueryParam<Uuid, true>,
    name: QueryParam<String, true>,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    FileService::rename_item(&claims.uid, &iid.into_inner(), &name.into_inner()).await?;
    res.render(Json(ResultData::<String>::new(
        "Completed rename",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "move item", body = ResultData<String>),
    )
)]
pub async fn move_item(
    move_item_dto: JsonBody<MoveItemDto>,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    FileService::move_item(&claims.uid, &move_item_dto.item_id, move_item_dto.parent_id).await?;
    res.render(Json(ResultData::<String>::new(
        "Completed move",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}
