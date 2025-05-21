use crate::config::Config;
use nacos_sdk::api::constants;
use nacos_sdk::api::naming::{NamingServiceBuilder, ServiceInstance};
use nacos_sdk::api::props::ClientProps;

pub async fn connect_nacos(
    app_name: impl Into<String> + Clone,
    namespace: impl Into<String>,
    config: &Config,
    port: u16,
) {
    let naming_service = NamingServiceBuilder::new(
        ClientProps::new()
            .server_addr(config.nacos.api.clone())
            .namespace(namespace)
            .app_name(app_name.clone())
            .auth_access_key(config.nacos.auth_username.clone())
            .auth_access_secret(config.nacos.auth_password.clone()),
    )
    .enable_auth_plugin_aliyun()
    .build()
    .unwrap();

    let service_instance = ServiceInstance {
        ip: config.nacos.this_server_ip.clone(),
        port: port as i32,
        ..Default::default()
    };

    let _register_instance_ret = naming_service
        .batch_register_instance(
            app_name.into(),
            Some(constants::DEFAULT_GROUP.to_string()),
            vec![service_instance],
        )
        .await;
}
