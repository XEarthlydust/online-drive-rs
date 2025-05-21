use chrono::{DateTime, Utc};
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(ToSchema, Deserialize, Serialize, Debug, Clone)]
pub struct ResultData<T: Serialize> {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub status: u16,
    pub data: Option<T>,
}

#[derive(Debug)]
pub enum ResultCode {
    Success = 2000,

    InnerError = 5000,
    DatabaseError = 5001,
    TokenCreateError = 5002,
    HashError = 5003,
    PayloadCreateError = 5004,
    MinioClientError = 5005,

    MissingField = 4000,

    TokenInvalid = 4010,
    UserPasswordMismatch = 4011,
    MissingToken = 4012,
    PermissionDenied = 4013,
    PayloadInvalid = 4014,
    PathOrNameError = 4015,
    CannotDeleteFile = 4016,
    CannotDeleteFolder = 4017,
    ShareCodeMismatched = 4018,

    UserExists = 4090,
    UserNotExists = 4040,
    FileNotExists = 4041,
    ItemNotExists = 4042,
    ShareFileNotFound = 4043,
}

impl<T: Serialize> ResultData<T> {
    pub fn new<M: Into<String>>(message: M, data: Option<T>, status: ResultCode) -> ResultData<T> {
        ResultData {
            timestamp: Utc::now(),
            message: message.into(),
            status: status as u16,
            data,
        }
    }
}
