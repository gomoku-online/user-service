#[cfg(test)]
mod test {
    use crate::test_util::oracle::test_uow_manager::{get_or_create_uow_manager, Connection};
    use crate::test_util::oracle::utils::test_with_transaction;
    use mockall::mock;
    use rand::distr::Alphanumeric;
    use rand::Rng;
    use shared_kernel::enums::auth_provider::AuthProvider;
    use shared_kernel::value_object::auth_id::AuthId;
    use std::sync::Arc;
    use user_service::user::application::port::outbound::persistence::create_user_repository::CreateUserRepository;
    use user_service::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
    use user_service::shared::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
    use user_service::shared::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;
    use user_service::user::domain::user::user::User;
    use user_service::user::domain::user::user_nickname::UserNickname;
    use user_service::user::infrastructure::persistence::oracle::adapter::create_user_oracle_adapter::CreateUserOracleAdapter;
    use user_service::user::infrastructure::persistence::oracle::constraint::create_user_constraint_mapper::CreateUserConstraintMapper;
    use user_service::shared::infrastructure::persistence::oracle::error_parser::OracleErrorParser;
    use user_service::shared::infrastructure::uow::uow_sync_manager::UnitOfWorkSyncManager;
    use uuid::Uuid;

    mock! {
        pub UnitOfWorkSyncManager {}

        impl UnitOfWorkSyncManager for UnitOfWorkSyncManager {
            type Conn = Connection;

            fn get_connection_with_uow(
                &self,
                uow: Arc<dyn UnitOfWork>,
            ) -> Result<Arc<tokio::sync::Mutex<Connection>>, UnitOfWorkError>;

            fn get_connection(&self) -> Result<Arc<tokio::sync::Mutex<Connection>>, UnitOfWorkError>;
        }
    }

    #[tokio::test]
    async fn return_error_when_get_connection_return_error() {
        // given
        let error_mocked_sync_manager = get_error_mocked_sync_manager();
        let create_user_adapter = Arc::new(get_mocked_create_user_oracle_adapter(
            error_mocked_sync_manager,
        ));
        let new_user = get_random_user();

        // when
        let result = create_user_adapter.create(None, new_user.clone()).await;

        // then
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_user_success() {
        test_with_transaction(|tx| async move {
            // given
            let create_user_adapter = get_create_user_adapter().await;
            let new_user = get_random_user();

            // when
            let result = create_user_adapter.create(Some(tx), new_user.clone()).await;

            // then
            assert!(result.is_ok());
            let saved_user = result.unwrap();
            assert!(saved_user.get_user_id().is_some());
            assert_eq!(saved_user.get_user_nickname(), new_user.get_user_nickname());
            assert_eq!(saved_user.get_active(), new_user.get_active());

            let saved_auth = saved_user.get_user_auths().get(0).unwrap();
            let new_auth = new_user.get_user_auths().get(0).unwrap();
            assert_eq!(saved_auth.get_auth_provider(), new_auth.get_auth_provider());
            assert_eq!(saved_auth.get_auth_id(), new_auth.get_auth_id());
        })
        .await;
    }

    #[tokio::test]
    async fn when_user_nickname_already_exists_return_error() {
        test_with_transaction(|tx| async move {
            // given
            let create_user_adapter = get_create_user_adapter().await;
            let nickname = get_random_user_nickname();
            let saved_user = get_random_user_by_nickname(nickname.clone());
            let new_user = get_random_user_by_nickname(nickname.clone());

            create_user_adapter
                .create(Some(tx.clone()), saved_user)
                .await
                .unwrap();

            // when
            let result = create_user_adapter.create(Some(tx), new_user).await;

            // then
            assert!(result.is_err());
        })
        .await;
    }

    fn get_error_mocked_sync_manager() -> Arc<MockUnitOfWorkSyncManager> {
        let mut mocked_manager = MockUnitOfWorkSyncManager::new();

        mocked_manager
            .expect_get_connection()
            .returning(|| Err(UnitOfWorkError::NotFound(UnitOfWorkId::new())));

        mocked_manager
            .expect_get_connection_with_uow()
            .returning(|tx| Err(UnitOfWorkError::NotFound(tx.get_id().clone())));

        Arc::new(mocked_manager)
    }

    fn get_mocked_create_user_oracle_adapter(
        mocked_manager: Arc<MockUnitOfWorkSyncManager>,
    ) -> CreateUserOracleAdapter<MockUnitOfWorkSyncManager, CreateUserConstraintMapper> {
        let constraint_mapper = Arc::new(CreateUserConstraintMapper);
        let oracle_error_parser = Arc::new(OracleErrorParser);
        CreateUserOracleAdapter::new(mocked_manager, constraint_mapper, oracle_error_parser)
    }

    async fn get_create_user_adapter() -> Arc<dyn CreateUserRepository> {
        let uow_manager = get_or_create_uow_manager().await;
        let constraint_mapper = Arc::new(CreateUserConstraintMapper);
        let oracle_error_parser = Arc::new(OracleErrorParser);

        let create_user_adapter = Arc::new(CreateUserOracleAdapter::new(
            uow_manager,
            constraint_mapper,
            oracle_error_parser,
        ));

        Arc::new(create_user_adapter)
    }

    fn get_random_auth_id() -> AuthId {
        AuthId::new(Uuid::new_v4().to_string())
    }

    fn get_random_user() -> User {
        User::create(
            get_random_user_nickname(),
            AuthProvider::KEYCLOAK,
            get_random_auth_id(),
        )
    }

    fn get_random_user_by_nickname(user_nickname: UserNickname) -> User {
        User::create(user_nickname, AuthProvider::KEYCLOAK, get_random_auth_id())
    }

    fn get_random_user_nickname() -> UserNickname {
        let length = 30;
        let random_string: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();
        UserNickname::new(random_string)
    }
}
