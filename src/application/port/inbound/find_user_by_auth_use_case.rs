use crate::application::dtos::user_dto::UserSummaryDto;
use async_trait::async_trait;
use getset::{CopyGetters, Getters};
use serde::Deserialize;
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use std::fmt::Debug;
use thiserror::Error;
use validator::Validate;

#[async_trait]
pub trait FindUserByAuthQueryUseCase {
    async fn execute(
        &self,
        query: FindUserByAuthQuery,
    ) -> Result<Option<UserSummaryDto>, FindUserByAuthError>;
}

#[derive(Debug, Validate, Getters, CopyGetters, Deserialize)]
pub struct FindUserByAuthQuery {
    #[getset(get = "pub with_prefix")]
    auth_provider: AuthProvider,
    #[getset(get = "pub with_prefix")]
    auth_id: AuthId,
}

impl FindUserByAuthQuery {
    pub fn new(auth_provider: AuthProvider, auth_id: AuthId) -> Self {
        Self {
            auth_provider,
            auth_id,
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FindUserByAuthError {
    #[error("예기치 못한 에러가 발생하였습니다: {0}")]
    Unknown(String),
}
