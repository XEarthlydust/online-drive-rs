use crate::config::Config;
use crate::util::database::init_rbpool;
use crate::util::minio::generate_client;
use argon2::{Algorithm, Argon2, Params, Version};
use aws_sdk_s3::Client;
use rbatis::RBatis;
use reqwest;
use std::sync::{LazyLock, OnceLock};

pub static CONTEXT: LazyLock<ServiceContext> = LazyLock::new(|| ServiceContext::default());

pub struct ServiceContext {
    pub rb: RBatis,
    pub config: Config,
    pub minio_client: OnceLock<Client>,
    pub argon2: Argon2<'static>,
    pub req_client: reqwest::Client,
}

impl ServiceContext {
    pub async fn init_database(&self) {
        init_rbpool(&self.config, &self.rb).await;
    }

    pub async fn init_minio(&self) {
        let client = generate_client(&self.config).await;
        self.minio_client.set(client).unwrap();
    }

    pub fn get_minio(&self) -> &Client {
        self.minio_client.get().expect("MinIO not initialized")
    }
}

impl Default for ServiceContext {
    fn default() -> Self {
        let config = Config::init_config();
        ServiceContext {
            rb: {
                let rb = RBatis::new();
                rb
            },
            config,
            minio_client: OnceLock::new(),
            argon2: Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default()),
            req_client: reqwest::Client::builder()
                .no_gzip()
                .no_deflate()
                .no_brotli()
                .build()
                .unwrap(),
        }
    }
}

#[macro_export]
macro_rules! db_pool {
    () => {
        &$crate::context::CONTEXT.rb
    };
}

#[macro_export]
macro_rules! argon2_client {
    () => {
        &$crate::context::CONTEXT.argon2
    };
}

#[macro_export]
macro_rules! minio_client {
    () => {
        &$crate::context::CONTEXT.minio_client.get().unwrap()
    };
}

#[macro_export]
macro_rules! config {
    () => {
        &$crate::context::CONTEXT.config
    };
}

#[macro_export]
macro_rules! req_client {
    () => {
        &$crate::context::CONTEXT.req_client
    };
}
