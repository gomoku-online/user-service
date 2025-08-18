use crate::user::application::dtos::user_dto::UserSummaryDto;
use crate::user::application::port::inbound::find_user_by_auth_use_case::{
    FindUserByAuthError, FindUserByAuthQuery, FindUserByAuthQueryUseCase,
};
use crate::user::application::port::outbound::persistence::find_user_by_auth_repository::{
    FindUserByAuthRepository, FindUserByAuthRepositoryError,
};
use crate::shared::application::port::outbound::uow::unit_of_work_manager::UnitOfWorkManager;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
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
                query.get_auth_provider().clone(),
                query.get_auth_id().clone(),
            )
            .await;

        let user_opt = match user_opt_result {
            Ok(user_opt) => user_opt,
            Err(e) => {
                return match e {
                    FindUserByAuthRepositoryError::Unknown(err_msg) => {
                        Err(FindUserByAuthError::Unknown(err_msg))
                    }
                };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::application::port::inbound::find_user_by_auth_use_case::FindUserByAuthQuery;
    use crate::user::application::port::outbound::persistence::find_user_by_auth_repository::{
        FindUserByAuthRepository, FindUserByAuthRepositoryError,
    };
    use crate::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
    use crate::shared::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
    use crate::shared::application::port::outbound::uow::unit_of_work_manager::UnitOfWorkManager;
    use crate::user::domain::user::user::User;
    use crate::user::domain::user::user_nickname::UserNickname;
    use async_trait::async_trait;
    use mockall::mock;
    use shared_kernel::enums::auth_provider::AuthProvider;
    use shared_kernel::value_object::auth_id::AuthId;
    use shared_kernel::value_object::user_id::UserId;
    use std::sync::Arc;

    mock! {
        #[derive(Debug)]
        pub UnitOfWorkManager {}

        #[async_trait]
        impl UnitOfWorkManager for UnitOfWorkManager {
            async fn begin(&self) -> Result<Arc<dyn UnitOfWork>, UnitOfWorkError>;
            async fn commit(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError>;
            async fn rollback(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError>;
        }
    }

    mock! {
        #[derive(Debug)]
        pub FindUserByAuthRepository {}

        #[async_trait]
        impl FindUserByAuthRepository for FindUserByAuthRepository {
            async fn find(
                &self,
                uow: Option<Arc<dyn UnitOfWork>>,
                auth_provider: AuthProvider,
                auth_id: AuthId,
            ) -> Result<Option<User>, FindUserByAuthRepositoryError>;
        }
    }

    struct Mocks {
        uow_manager: MockUnitOfWorkManager,
        repo: MockFindUserByAuthRepository,
    }

    fn setup() -> Mocks {
        Mocks {
            uow_manager: MockUnitOfWorkManager::new(),
            repo: MockFindUserByAuthRepository::new(),
        }
    }

    #[tokio::test]
    async fn should_return_user_summary_dto_when_user_is_found() {
        // Given
        let mut mocks = setup();
        let query = FindUserByAuthQuery::new(
            AuthProvider::KEYCLOAK,
            AuthId::new("found-user".to_string()),
        );
        let user_id = UserId::new(1);
        let nickname = UserNickname::new("test_user".to_string());

        let found_user = User::new(Some(user_id), nickname.clone(), true, vec![]);

        mocks
            .repo
            .expect_find()
            .times(1)
            .returning(move |_, _, _| Ok(Some(found_user.clone())));

        let service = FindUserByAuthService::new(Arc::new(mocks.uow_manager), Arc::new(mocks.repo));

        // When
        let result = service.execute(query).await;

        // Then
        assert!(result.is_ok());
        let user_dto_opt = result.unwrap();
        assert!(user_dto_opt.is_some());

        let user_dto = user_dto_opt.unwrap();
        assert_eq!(user_dto.get_user_id(), user_id);
        assert_eq!(user_dto.get_user_nickname(), &nickname);
        assert_eq!(user_dto.get_active(), true);
    }

    #[tokio::test]
    async fn should_return_none_when_user_is_not_found() {
        // Given
        let mut mocks = setup();
        let query = FindUserByAuthQuery::new(
            AuthProvider::KEYCLOAK,
            AuthId::new("not-found-user".to_string()),
        );

        mocks
            .repo
            .expect_find()
            .times(1)
            .returning(|_, _, _| Ok(None));

        let service = FindUserByAuthService::new(Arc::new(mocks.uow_manager), Arc::new(mocks.repo));

        // When
        let result = service.execute(query).await;

        // Then
        assert!(result.is_ok());
        let user_dto_opt = result.unwrap();
        assert!(user_dto_opt.is_none());
    }

    #[tokio::test]
    async fn should_return_error_when_repository_fails() {
        // Given
        let mut mocks = setup();
        let query = FindUserByAuthQuery::new(
            AuthProvider::KEYCLOAK,
            AuthId::new("error-user".to_string()),
        );
        let error_message = "DB 연결 실패".to_string();

        mocks.repo.expect_find().times(1).returning(move |_, _, _| {
            Err(FindUserByAuthRepositoryError::Unknown(
                error_message.clone(),
            ))
        });

        let service = FindUserByAuthService::new(Arc::new(mocks.uow_manager), Arc::new(mocks.repo));

        // When
        let result = service.execute(query).await;

        // Then
        assert!(result.is_err());
        match result.err().unwrap() {
            FindUserByAuthError::Unknown(msg) => assert_eq!(msg, "DB 연결 실패"),
        }
    }

    #[tokio::test]
    async fn should_return_error_when_found_user_has_no_id() {
        // Given
        let mut mocks = setup();
        let query = FindUserByAuthQuery::new(
            AuthProvider::KEYCLOAK,
            AuthId::new("no-id-user".to_string()),
        );
        let nickname = UserNickname::new("no_id_user".to_string());

        let malformed_user = User::new(None, nickname.clone(), true, vec![]);

        mocks
            .repo
            .expect_find()
            .times(1)
            .returning(move |_, _, _| Ok(Some(malformed_user.clone())));

        let service = FindUserByAuthService::new(Arc::new(mocks.uow_manager), Arc::new(mocks.repo));

        // When
        let result = service.execute(query).await;

        // Then
        assert!(result.is_err());
        match result.err().unwrap() {
            FindUserByAuthError::Unknown(msg) => assert_eq!(msg, "유저 ID가 존재하지 않습니다."),
        }
    }
}
