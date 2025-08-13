use crate::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use crate::domain::user::user::User;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

#[async_trait]
pub trait CreateUserRepository: Debug + Send + Sync {
    async fn create(
        &self,
        tx: Option<Arc<dyn UnitOfWork>>,
        user: User,
    ) -> Result<User, CreateUserRepositoryError>;
}

#[derive(Debug, Error)]
pub enum CreateUserRepositoryError {
    #[error("이미 등록된 사용자입니다")]
    AlreadyRegistered,
    #[error("이미 존재하는 닉네임입니다")]
    AlreadyExistNickname,
    #[error("예기치 못한 에러가 발생하였습니다: {0}")]
    Unknown(String),
}

impl From<UnitOfWorkError> for CreateUserRepositoryError {
    fn from(value: UnitOfWorkError) -> Self {
        CreateUserRepositoryError::Unknown(value.to_string())
    }
}
