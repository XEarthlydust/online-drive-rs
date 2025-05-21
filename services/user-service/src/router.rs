use salvo::prelude::Router;
use crate::handler::*;
use common::handler::{admin_middleware, auth_middleware};

pub fn all_router() -> Router {
    Router::with_path("api").push(
        Router::with_path("user")
            .push(Router::with_path("login").post(login))
            .push(Router::with_path("register").put(register))
            .push(Router::with_path("info-public{user_id}").get(user_info_pubic))
            .push(Router::with_path("info").hoop(auth_middleware).get(user_info))
            .push(Router::with_path("update-password").hoop(auth_middleware).post(change_password))
            .push(Router::with_path("check-token").hoop(auth_middleware).get(check_token))
            .push(Router::with_path("update-info").hoop(auth_middleware).post(update_userinfo))
            .push(Router::with_path("update-avatar").hoop(auth_middleware).put(set_avatar))
            .push(Router::with_path("delete-user{user_id}").hoop(auth_middleware).hoop(admin_middleware).post(delete_user))
            .push(Router::with_path("get-all-user").hoop(auth_middleware).hoop(admin_middleware).post(get_users))
            .push(Router::with_path("set-user-size").hoop(auth_middleware).hoop(admin_middleware).post(set_user_max_size))
    )
}