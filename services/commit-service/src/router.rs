use salvo::prelude::Router;
use crate::handler::*;
use common::handler::{auth_middleware};

pub fn all_router() -> Router {
    Router::with_path("api").push(
        Router::with_path("commit")
            .push(Router::with_path("get").get(get_commit))
            .push(Router::with_path("delete").hoop(auth_middleware).delete(delete_commit))
            .push(Router::with_path("create").hoop(auth_middleware).put(create_commit))
    )
}