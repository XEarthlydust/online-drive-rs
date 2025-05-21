mod handler;
mod router;
mod service;

use crate::router::all_router;
use common::config;
use common::context::*;
use common::util::nacos::connect_nacos;
use common::util::router::openapi;
use salvo::logging::Logger;
use salvo::prelude::*;
use tokio::signal;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let config = &CONTEXT.config;
    CONTEXT.init_database().await;
    CONTEXT.init_minio().await;

    let router = openapi(
        Router::new().push(all_router()),
        config,
        env!("CARGO_PKG_NAME").into(),
        env!("CARGO_PKG_VERSION").into(),
    );
    let acceptor = TcpListener::new(format!("{}:{}", config.port.bind_ip, config.port.file))
        .bind()
        .await;
    let service = Service::new(router).hoop(Logger::new());
    let server = Server::new(acceptor);
    let handle = server.handle();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
        handle.stop_graceful(None);
    });
    connect_nacos(env!("CARGO_PKG_NAME"), "", config!(), config!().port.file).await;
    server.serve(service).await;
}
