use crate::config;
use crate::config::Config;
use crate::module::error::AppError;
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::Client;
use std::time::Duration;

pub async fn generate_client(config: &Config) -> Client {
    let region = Region::new("local");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(&config.minio.endpoint) // MinIO 服务器地址
        .region(region) // MinIO 默认区域
        .credentials_provider(Credentials::new(
            &config.minio.access_key, // 访问密钥
            &config.minio.secret_key, // 秘密密钥
            None,                     // 令牌 (可选)
            None,                     // 过期时间 (可选)
            "minio",                  // 提供者名称
        ))
        .load()
        .await;
    Client::new(&config)
}

pub async fn generate_upload_id(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<String, AppError> {
    let response = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;
    response.upload_id.ok_or(AppError::MinioClientError(
        "Cannot get upload id".to_string(),
    ))
}

pub async fn generate_part_upload_url(
    client: &Client,
    bucket: &str,
    key: &str,
    part_number: i64,
    upload_id: String,
) -> Result<String, AppError> {
    let presigned_request = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .part_number(part_number as i32)
        .presigned(
            PresigningConfig::expires_in(Duration::from_secs(config!().upload.part_exp_min * 60))
                .map_err(|e| AppError::InnerError(e.to_string()))?,
        )
        .await?;
    Ok(presigned_request.uri().to_string())
}

pub async fn generate_download_url(
    client: &Client,
    bucket: &str,
    key: &str,
    file_name: &str,
) -> Result<String, AppError> {
    let presigned = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .response_content_disposition(format!("attachment; filename=\"{}\"", file_name))
        .presigned(
            PresigningConfig::expires_in(Duration::from_secs(14400))
                .map_err(|e| AppError::InnerError(e.to_string()))?,
        )
        .await?;
    Ok(presigned.uri().to_string())
}

pub async fn complete_upload(
    client: &Client,
    bucket: &str,
    key: &str,
    upload_id: &String,
    parts: Vec<CompletedPart>,
) -> Result<(), AppError> {
    let completed = CompletedMultipartUpload::builder()
        .set_parts(Some(parts))
        .build();
    client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .multipart_upload(completed)
        .send()
        .await?;
    Ok(())
}
