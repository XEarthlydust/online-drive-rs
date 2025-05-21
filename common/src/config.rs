use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool_size: u64,
    pub timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Port {
    pub bind_ip: String,
    pub user: u16,
    pub file: u16,
    pub share: u16,
    pub commit: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nacos {
    pub api: String,
    pub auth_username: String,
    pub auth_password: String,
    pub this_server_ip: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub debug: bool,
    pub log_root: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenApi {
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinIO {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub avatar_bucket: String,
    pub file_bucket: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    pub exp_min: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Upload {
    pub part_exp_min: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sonyflake {
    pub id: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database: Database,
    pub port: Port,
    pub log: Log,
    pub nacos: Nacos,
    pub openapi: OpenApi,
    pub minio: MinIO,
    pub jwt: Jwt,
    pub upload: Upload,
    pub page: Page,
    pub sonyflake: Sonyflake,
}

impl Config {
    pub fn init_config() -> Self {
        let config_path = "config.toml";
        match fs::read_to_string(config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => config,
                Err(_) => {
                    let default = Self::default();
                    Self::write_default_config(config_path, &default);
                    default
                }
            },
            Err(_) => {
                let default = Self::default();
                Self::write_default_config(config_path, &default);
                default
            }
        }
    }

    fn write_default_config(path: &str, config: &Config) {
        if let Ok(toml_str) = toml::to_string_pretty(config) {
            let _ = File::create(path).and_then(|mut file| file.write_all(toml_str.as_bytes()));
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database: Database {
                url: "postgres://username:password@localhost/online-drive-base".into(),
                pool_size: 20,
                timeout: 5,
            },
            port: Port {
                bind_ip: "0.0.0.0".into(),
                user: 8001,
                file: 8002,
                share: 8003,
                commit: 8004,
            },
            log: Log {
                debug: false,
                log_root: "/log".to_string(),
            },
            openapi: OpenApi { enable: true },
            minio: MinIO {
                endpoint: "http://localhost:9000".into(),
                access_key: "admin".into(),
                secret_key: "admin".into(),
                region: "local".to_string(),
                avatar_bucket: "avatar".to_string(),
                file_bucket: "file".to_string(),
            },
            jwt: Jwt { exp_min: 600 },
            upload: Upload { part_exp_min: 10 },
            page: Page { size: 10 },
            sonyflake: Sonyflake { id: 1 },
            nacos: Nacos {
                api: "127.0.0.1:8848".to_string(),
                auth_username: "KEY".to_string(),
                auth_password: "VALUE".to_string(),
                this_server_ip: "ip.youragent.thisserver".to_string(),
            },
        }
    }
}
