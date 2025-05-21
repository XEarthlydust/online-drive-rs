use crate::config::Config;
use salvo::oapi::OpenApi;
use salvo::prelude::SwaggerUi;
use salvo::Router;
use tracing::info;

pub fn openapi(router: Router, config: &Config, title: String, version: String) -> Router {
    let base_router = router;
    let doc = OpenApi::new(title, version).merge_router(&base_router);
    if config.openapi.enable {
        info!("Enable open api");
        base_router
            .unshift(doc.into_router("/api-doc/openapi.json"))
            .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"))
    } else {
        base_router
    }
}
