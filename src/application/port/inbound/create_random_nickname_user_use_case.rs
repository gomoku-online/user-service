use crate::application::dtos::user_dto::UserSummaryDto;
use crate::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use crate::domain::user::user_nickname::UserNickname;
use async_trait::async_trait;
use getset::{CopyGetters, Getters};
use serde::Deserialize;
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use std::fmt::Debug;
use thiserror::Error;
use validator::Validate;

#[async_trait]
pub trait CreateRandomNicknameUserUseCase {
    async fn execute(
        &self,
        command: CreateRandomNicknameUserCommand,
    ) -> Result<UserSummaryDto, CreateRandomNicknameUserError>;
}

#[derive(Debug, Validate, CopyGetters, Getters, Deserialize)]
pub struct CreateRandomNicknameUserCommand {
    #[getset(get_copy = "pub with_prefix")]
    auth_provider: AuthProvider,
    #[getset(get = "pub with_prefix")]
    auth_id: AuthId,
}

impl CreateRandomNicknameUserCommand {
    pub fn new(auth_provider: AuthProvider, auth_id: AuthId) -> Self {
        Self {
            auth_provider,
            auth_id,
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CreateRandomNicknameUserError {
    #[error("이미 등록된 사용자입니다: {0}")]
    AlreadyRegistered(String),
    #[error("고유 닉네임 생성에 실패했습니다: {0}")]
    NicknameGenerationFailed(String),
    #[error("예기치 못한 에러가 발생하였습니다: {0}")]
    Unknown(String),
}

impl From<UnitOfWorkError> for CreateRandomNicknameUserError {
    fn from(value: UnitOfWorkError) -> Self {
        CreateRandomNicknameUserError::Unknown(value.to_string())
    }
}
