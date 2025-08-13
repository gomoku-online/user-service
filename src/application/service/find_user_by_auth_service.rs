use crate::application::dtos::user_dto::UserSummaryDto;
use crate::application::port::inbound::find_user_by_auth_use_case::{
    FindUserByAuthError, FindUserByAuthQuery, FindUserByAuthQueryUseCase,
};
use crate::application::port::outbound::persistence::find_user_by_auth_repository::{FindUserByAuthRepository, FindUserByAuthRepositoryError};
use crate::application::port::outbound::uow::unit_of_work_manager::UnitOfWorkManager;
use async_trait::async_trait;
use std::sync::Arc;
use crate::domain::user::user::User;

#[derive(Debug, Clone)]
pub struct FindUserByAuthService<T, P>
where
    T: UnitOfWorkManager + ?Sized,
    P: FindUserByAuthRepository + ?Sized,
{
    uow_manager: Arc<T>,
    find_user_by_auth_repository: Arc<P>,
}

impl<T, P> FindUserByAuthService<T, P>
where
    T: UnitOfWorkManager + ?Sized,
    P: FindUserByAuthRepository + ?Sized,
{
    pub fn new(uow_manager: Arc<T>, find_user_by_auth_repository: Arc<P>) -> Self {
        Self {
            uow_manager,
            find_user_by_auth_repository,
        }
    }
}

#[async_trait]
impl<T, P> FindUserByAuthQueryUseCase for FindUserByAuthService<T, P>
where
    T: UnitOfWorkManager + ?Sized,
    P: FindUserByAuthRepository + ?Sized,
{
    async fn execute(
        &self,
        query: FindUserByAuthQuery,
    ) -> Result<Option<UserSummaryDto>, FindUserByAuthError> {
        let user_opt_result = self
            .find_user_by_auth_repository
            .find(
                None,
                query.get_auth_id().clone(),
                query.get_auth_provider().clone(),
            )
            .await;

        let user_opt = match user_opt_result {
            Ok(user_opt) => user_opt,
            Err(e) => return match e {
                FindUserByAuthRepositoryError::Unknown(err_msg) => Err(FindUserByAuthError::Unknown(err_msg))
            }
        };

        if let Some(user) = user_opt {
            let user_id = user.get_user_id().ok_or_else(|| {
                FindUserByAuthError::Unknown("유저 ID가 존재하지 않습니다.".to_string())
            })?;

            let user_nickname = user.get_user_nickname().clone();
            let active = user.get_active();

            let dto = UserSummaryDto::new(user_id, user_nickname, active);

            Ok(Some(dto))
        } else {
            Ok(None)
        }
    }
}
