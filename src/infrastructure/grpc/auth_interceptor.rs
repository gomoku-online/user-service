use crate::infrastructure::auth::authentication::Authentication;
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use std::str::FromStr;
use tonic::service::Interceptor;
use tonic::{Request, Status};

const AUTH_PROVIDER_HEADER_NAME: &str = "x-auth-provider";
const AUTH_ID_HEADER_NAME: &str = "x-auth-id";

#[derive(Debug, Clone)]
pub struct AuthInterceptor;

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let metadata = req.metadata();

        let auth_provider_str = metadata
            .get(AUTH_PROVIDER_HEADER_NAME)
            .ok_or_else(|| {
                Status::unauthenticated(format!("Missing {} header", AUTH_PROVIDER_HEADER_NAME))
            })?
            .to_str()
            .map_err(|_| {
                Status::unauthenticated(format!("Invalid {} format", AUTH_PROVIDER_HEADER_NAME))
            })?;

        let auth_provider = AuthProvider::from_str(auth_provider_str)
            .map_err(|_| Status::unauthenticated("Invalid auth provider"))?;

        let auth_id_str = metadata
            .get(AUTH_ID_HEADER_NAME)
            .ok_or_else(|| {
                Status::unauthenticated(format!("Missing {} header", AUTH_ID_HEADER_NAME))
            })?
            .to_str()
            .map_err(|_| {
                Status::unauthenticated(format!("Invalid {} format", AUTH_ID_HEADER_NAME))
            })?
            .to_string();

        let auth_id = AuthId::new(auth_id_str);

        let authentication = Authentication::new(auth_provider, auth_id);

        req.extensions_mut().insert(authentication);

        Ok(req)
    }
}
