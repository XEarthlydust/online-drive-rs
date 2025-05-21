use crate::config::Config;
use rbatis::RBatis;
use rbdc_pg::PostgresDriver;

pub async fn init_rbpool(config: &Config, rb: &RBatis) {
    rb.link(PostgresDriver {}, config.database.url.as_str())
        .await
        .unwrap();
    let pool = rb.get_pool().unwrap();
    pool.set_max_open_conns(config.database.pool_size).await;
}
