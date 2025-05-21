use crate::module::error::AppError;
use crate::util::jwt::{validate_jwt, Claims};
use crate::util::result::{ResultCode, ResultData};
use salvo::oapi::extract::HeaderParam;
use salvo::prelude::Json;
use salvo::Writer;
use salvo::{handler, Depot, Response};

#[handler]
pub async fn auth_middleware(
    token: HeaderParam<String, true>,
    depot: &mut Depot,
) -> Result<(), AppError> {
    let token = token.into_inner();
    let value = validate_jwt(token.as_str())?;
    depot.insert("claims", value);
    Ok(())
}

#[handler]
pub async fn admin_middleware(depot: &mut Depot) -> Result<(), AppError> {
    let claims = depot
        .get::<Claims>("claims")
        .map_err(|_e| AppError::MissingToken)?;
    if claims.user_role != "admin" {
        return Err(AppError::PermissionDenied);
    }
    Ok(())
}

#[handler]
pub async fn try_jwt(res: &mut Response) -> Result<(), ()> {
    res.render(Json(ResultData::new("Ok", None::<()>, ResultCode::Success)));
    Ok(())
}
