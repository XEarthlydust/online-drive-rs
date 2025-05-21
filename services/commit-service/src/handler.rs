use crate::service::commit_service::CommitService;
use common::config;
use common::module::commit::Commit;
use common::module::error::AppError;
use common::util::jwt::Claims;
use common::util::result::{ResultCode, ResultData};
use rbatis::Page;
use salvo::oapi::extract::JsonBody;
use salvo::oapi::ToSchema;
use salvo::prelude::*;
use salvo::Response;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
struct CommitsDto {
    share_id: Uuid,
    code: Option<String>,
    page: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
struct CreateCommitDto {
    share_id: Uuid,
    code: Option<String>,
    text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
struct DeleteCommitDto {
    share_id: Uuid,
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "get commits", body = ResultData<Vec<Commit>>),
    )
)]
pub async fn get_commit(
    commits_dto: JsonBody<CommitsDto>,
    res: &mut Response,
) -> Result<StatusCode, AppError> {
    let commits = CommitService::get_commit_page(
        &commits_dto.share_id,
        commits_dto.code.clone(),
        commits_dto.page,
        config!().page.size,
    )
    .await?;
    res.render(Json(ResultData::<Page<Commit>>::new(
        "Get success",
        Some(commits),
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "create commit", body = ResultData<String>),
    )
)]
pub async fn create_commit(
    create_commit_dto: JsonBody<CreateCommitDto>,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    CommitService::create_commit(
        &claims.uid,
        &create_commit_dto.share_id,
        create_commit_dto.code.clone(),
        create_commit_dto.text.clone(),
    )
    .await?;
    res.render(Json(ResultData::<Page<Commit>>::new(
        "Create success",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}

#[endpoint(
    status_codes(200),
    responses(
        (status_code = 200, description = "delete commit", body = ResultData<String>),
    )
)]
pub async fn delete_commit(
    delete_commit_dto: JsonBody<DeleteCommitDto>,
    res: &mut Response,
    depot: &mut Depot,
) -> Result<StatusCode, AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    CommitService::delete_commit(&delete_commit_dto.share_id, &claims.uid).await?;
    res.render(Json(ResultData::<Page<Commit>>::new(
        "Delete success",
        None,
        ResultCode::Success,
    )));
    Ok(StatusCode::OK)
}
