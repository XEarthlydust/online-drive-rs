use salvo::prelude::Router;
use crate::handler::*;
use common::handler::{auth_middleware};

pub fn all_router() -> Router {
    Router::with_path("api").push(
        Router::with_path("share")
            .push(Router::with_path("get-publicly").post(get_share_publicly))
            .push(Router::with_path("get-with-code").post(get_share_with_code))
            .push(Router::with_path("get-all").hoop(auth_middleware).get(get_user_shares))
            .push(Router::with_path("create").hoop(auth_middleware).put(create_share))
            .push(Router::with_path("save").hoop(auth_middleware).put(save_share))
            .push(Router::with_path("delete{sid}").hoop(auth_middleware).delete(delete_share))
    )
}