use crate::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::shared::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use crate::user::domain::user::user::User;
use async_trait::async_trait;
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

#[async_trait]
pub trait FindUserByAuthRepository: Debug + Send + Sync {
    async fn find(
        &self,
        uow: Option<Arc<dyn UnitOfWork>>,
        auth_provider: AuthProvider,
        auth_id: AuthId,
    ) -> Result<Option<User>, FindUserByAuthRepositoryError>;
}

#[derive(Debug, Error)]
pub enum FindUserByAuthRepositoryError {
    #[error("예기치 못한 에러가 발생하였습니다: {0}")]
    Unknown(String),
}

impl From<UnitOfWorkError> for FindUserByAuthRepositoryError {
    fn from(value: UnitOfWorkError) -> Self {
        FindUserByAuthRepositoryError::Unknown(value.to_string())
    }
}
