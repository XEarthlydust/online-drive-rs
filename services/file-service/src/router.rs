use salvo::prelude::Router;
use crate::handler::*;
use common::handler::{auth_middleware, check_size};

pub fn all_router() -> Router {
    Router::with_path("api").push(
        Router::with_path("file")
            .push(Router::with_path("upload-by-hash").hoop(auth_middleware).hoop(check_size).put(upload_by_hash))
            .push(Router::with_path("start-upload").hoop(auth_middleware).hoop(check_size).put(start_upload_file))
            .push(Router::with_path("upload-part").hoop(auth_middleware).put(upload_file_part))
            .push(Router::with_path("finish-upload").hoop(auth_middleware).hoop(check_size).post(finish_upload))
            .push(Router::with_path("mkdir").hoop(auth_middleware).put(make_logic_dir))
            .push(Router::with_path("get").hoop(auth_middleware).post(get_item))
            .push(Router::with_path("move").hoop(auth_middleware).post(move_item))
            .push(Router::with_path("download{**}").hoop(auth_middleware).get(download))
            .push(Router::with_path("delete{**}").hoop(auth_middleware).delete(delete))
            .push(Router::with_path("rename{**}").hoop(auth_middleware).post(rename))
    )
}