use crate::util::result::{ResultCode, ResultData};
use argon2::password_hash::Error as PasswordHashError;
use aws_sdk_s3::config::http::HttpResponse;
use aws_sdk_s3::error::SdkError;
use rbatis::Error as RbatisError;
use salvo::http::{StatusCode, StatusError};
use salvo::oapi::{EndpointOutRegister, ToSchema};
use salvo::prelude::Json;
use salvo::{async_trait, oapi, Depot, Request, Response, Writer};
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] RbatisError),

    #[error("Hashing error: {0}")]
    HashError(String),

    #[error("User account already exists")]
    UserExists,

    #[error("User account not found")]
    UserNotExists,

    #[error("Password or account not match")]
    UserPasswordMismatch,

    #[error("{0} error")]
    InnerError(String),

    #[error("Token invalid or expired")]
    TokenInvalid,

    #[error("Payload invalid or expired")]
    PayloadInvalid,

    #[error("Cannot create JWT")]
    TokenCreateError,

    #[error("Cannot create payload")]
    PayloadCreateError,

    #[error("Missing form field: {0}")]
    MissingField(String),

    #[error("Missing token")]
    MissingToken,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("File not exists")]
    FileNotExists,

    #[error("Item not exists")]
    ItemNotExists,

    #[error("Invalid path or filename")]
    PathOrNameError,

    #[error("Minio error: {0}")]
    MinioClientError(String),

    #[error("Cannot remove this file, because it is folder")]
    CannotDeleteFolder,

    #[error("Cannot remove this folder, because it is file")]
    CannotDeleteFile,

    #[error("Cannot found this file")]
    ShareFileNotFound,

    #[error("Cannot get this file")]
    ShareCodeMismatched,
}
impl From<PasswordHashError> for AppError {
    fn from(err: PasswordHashError) -> Self {
        AppError::HashError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        AppError::InnerError(error.to_string())
    }
}

impl<T> From<SdkError<T, HttpResponse>> for AppError {
    fn from(error: SdkError<T, HttpResponse>) -> Self {
        AppError::InnerError(error.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError::InnerError(error.to_string())
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        warn!("{:?}", self);
        let (code, status_code, message) = match self {
            AppError::InnerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResultCode::InnerError,
                format!("{}", self.to_string()),
            ),
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResultCode::DatabaseError,
                format!("{}", self.to_string()),
            ),
            AppError::TokenCreateError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResultCode::TokenCreateError,
                format!("{}", self.to_string()),
            ),
            AppError::PayloadCreateError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResultCode::PayloadCreateError,
                format!("{}", self.to_string()),
            ),
            AppError::HashError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResultCode::HashError,
                format!("{}", self.to_string()),
            ),
            AppError::MissingField(_) => (
                StatusCode::BAD_REQUEST,
                ResultCode::MissingField,
                format!("{}", self.to_string()),
            ),
            AppError::TokenInvalid => (
                StatusCode::UNAUTHORIZED,
                ResultCode::TokenInvalid,
                format!("{}", self.to_string()),
            ),
            AppError::PayloadInvalid => (
                StatusCode::UNAUTHORIZED,
                ResultCode::PayloadInvalid,
                format!("{}", self.to_string()),
            ),
            AppError::UserPasswordMismatch => (
                StatusCode::UNAUTHORIZED,
                ResultCode::UserPasswordMismatch,
                format!("{}", self.to_string()),
            ),
            AppError::UserExists => (
                StatusCode::CONFLICT,
                ResultCode::UserExists,
                format!("{}", self.to_string()),
            ),
            AppError::UserNotExists => (
                StatusCode::NOT_FOUND,
                ResultCode::UserNotExists,
                format!("{}", self.to_string()),
            ),
            AppError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                ResultCode::MissingToken,
                format!("{}", self.to_string()),
            ),
            AppError::PermissionDenied => (
                StatusCode::UNAUTHORIZED,
                ResultCode::PermissionDenied,
                format!("{}", self.to_string()),
            ),
            AppError::FileNotExists => (
                StatusCode::NOT_FOUND,
                ResultCode::FileNotExists,
                format!("{}", self.to_string()),
            ),
            AppError::ItemNotExists => (
                StatusCode::NOT_FOUND,
                ResultCode::ItemNotExists,
                format!("{}", self.to_string()),
            ),
            AppError::PathOrNameError => (
                StatusCode::BAD_REQUEST,
                ResultCode::PathOrNameError,
                format!("{}", self.to_string()),
            ),
            AppError::MinioClientError(_) => (
                StatusCode::BAD_REQUEST,
                ResultCode::MinioClientError,
                format!("{}", self.to_string()),
            ),
            AppError::CannotDeleteFile => (
                StatusCode::BAD_REQUEST,
                ResultCode::CannotDeleteFile,
                format!("{}", self.to_string()),
            ),
            AppError::CannotDeleteFolder => (
                StatusCode::BAD_REQUEST,
                ResultCode::CannotDeleteFolder,
                format!("{}", self.to_string()),
            ),
            AppError::ShareFileNotFound => (
                StatusCode::NOT_FOUND,
                ResultCode::ShareFileNotFound,
                format!("{}", self.to_string()),
            ),
            AppError::ShareCodeMismatched => (
                StatusCode::UNAUTHORIZED,
                ResultCode::ShareCodeMismatched,
                format!("{}", self.to_string()),
            ),
        };
        res.status_code(code);
        res.render(Json(ResultData::new(message, None::<()>, status_code)));
    }
}

impl EndpointOutRegister for AppError {
    fn register(components: &mut oapi::Components, operation: &mut oapi::Operation) {
        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            oapi::Response::new("Internal server error")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::NOT_FOUND.as_str(),
            oapi::Response::new("Not found")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            oapi::Response::new("Bad request")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::CONFLICT.as_str(),
            oapi::Response::new("Conflict")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::UNAUTHORIZED.as_str(),
            oapi::Response::new("Unauthorized")
                .add_content("application/json", StatusError::to_schema(components)),
        );
    }
}
