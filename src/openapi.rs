use utoipa::OpenApi;

use crate::api_contract::{LoginRequest, LoginResponse, SignupRequest};

#[derive(Debug, OpenApi)]
#[openapi(
    paths(
        crate::api_contract::signup_openapi,
        crate::api_contract::login_openapi
    ),
    components(schemas(SignupRequest, LoginRequest, LoginResponse))
)]
pub struct AuthApiDoc;
