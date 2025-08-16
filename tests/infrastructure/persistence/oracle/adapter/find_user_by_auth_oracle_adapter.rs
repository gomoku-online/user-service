#[cfg(test)]
mod test {
    use crate::test_util::oracle::test_uow_manager::get_or_create_uow_manager;
    use crate::test_util::oracle::test_uow_manager::Connection;
    use crate::test_util::oracle::utils::test_with_transaction;
    use mockall::mock;
    use rand::distr::Alphanumeric;
    use rand::{rng, Rng};
    use shared_kernel::enums::auth_provider::AuthProvider;
    use shared_kernel::value_object::auth_id::AuthId;
    use std::sync::Arc;
    use user_service::application::port::outbound::persistence::create_user_repository::CreateUserRepository;
    use user_service::application::port::outbound::persistence::find_user_by_auth_repository::FindUserByAuthRepository;
    use user_service::application::port::outbound::uow::unit_of_work::UnitOfWork;
    use user_service::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
    use user_service::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;
    use user_service::domain::user::user::User;
    use user_service::domain::user::user_nickname::UserNickname;
    use user_service::infrastructure::persistence::oracle::adapter::create_user_oracle_adapter::CreateUserOracleAdapter;
    use user_service::infrastructure::persistence::oracle::adapter::find_user_by_auth_oracle_adapter::FindUserByAuthOracleAdapter;
    use user_service::infrastructure::persistence::oracle::constraint::mapper::constraint_mapper::ConstraintMapper;
    use user_service::infrastructure::persistence::oracle::constraint::mapper::create_user_constraint_mapper::CreateUserConstraintMapper;
    use user_service::infrastructure::persistence::oracle::error_parser::OracleErrorParser;
    use user_service::infrastructure::uow::uow_sync_manager::UnitOfWorkSyncManager;
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
        let auth_provider = get_random_auth_provider();
        let auth_id = get_random_auth_id();

        let error_mocked_uow_manager = get_error_mocked_sync_manager();
        let find_user_by_auth_adapter =
            Arc::new(FindUserByAuthOracleAdapter::new(error_mocked_uow_manager));

        // when
        let result = find_user_by_auth_adapter
            .find(None, auth_provider, auth_id)
            .await;

        // then
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn return_none_when_user_saved_in_not_committed_diff_transaction() {
        // given
        let created_user = get_random_user();
        let user_auth = created_user.get_user_auths().get(0).unwrap();
        let auth_provider = user_auth.get_auth_provider();
        let auth_id = user_auth.get_auth_id().clone();
        let find_user_by_auth_adapter = get_find_user_by_auth_adapter().await;
        let create_user_adapter = get_create_user_adapter().await;

        test_with_transaction(|tx| async move {
            // when
            create_user_adapter
                .create(Some(tx.clone()), created_user.clone())
                .await
                .unwrap();

            let result = find_user_by_auth_adapter
                .find(None, auth_provider, auth_id)
                .await;

            // then
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn return_user_info_when_user_exists() {
        // given
        let created_user = get_random_user();
        let user_auth = created_user.get_user_auths().get(0).unwrap();
        let auth_provider = user_auth.get_auth_provider();
        let auth_id = user_auth.get_auth_id().clone();
        let find_user_by_auth_adapter = get_find_user_by_auth_adapter().await;
        let create_user_adapter = get_create_user_adapter().await;

        test_with_transaction(|tx| async move {
            // when
            create_user_adapter
                .create(Some(tx.clone()), created_user.clone())
                .await
                .unwrap();

            let result = find_user_by_auth_adapter
                .find(Some(tx.clone()), auth_provider, auth_id)
                .await;

            // then
            assert!(result.is_ok());
            assert!(result.unwrap().is_some());
        })
        .await;
    }

    #[tokio::test]
    async fn return_none_when_user_not_exists() {
        // given
        let created_user = get_random_user();
        let user_auth = created_user.get_user_auths().get(0).unwrap();
        let auth_provider = user_auth.get_auth_provider();
        let auth_id = user_auth.get_auth_id().clone();
        let find_user_by_auth_adapter = get_find_user_by_auth_adapter().await;
        let create_user_adapter = get_create_user_adapter().await;

        test_with_transaction(|tx| async move {
            // when
            let result = find_user_by_auth_adapter
                .find(Some(tx.clone()), auth_provider, auth_id)
                .await;

            // then
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        })
        .await;
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

    async fn get_find_user_by_auth_adapter() -> Arc<dyn FindUserByAuthRepository> {
        let uow_manager = get_or_create_uow_manager().await;

        let find_user_by_auth_adapter = Arc::new(FindUserByAuthOracleAdapter::new(uow_manager));

        Arc::new(find_user_by_auth_adapter)
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

    fn get_random_auth_id() -> AuthId {
        AuthId::new(Uuid::new_v4().to_string())
    }

    fn get_random_auth_provider() -> AuthProvider {
        AuthProvider::KEYCLOAK
    }

    fn get_random_user() -> User {
        User::create(
            get_random_user_nickname(),
            AuthProvider::KEYCLOAK,
            get_random_auth_id(),
        )
    }

    fn get_random_user_nickname() -> UserNickname {
        let length = 30;

        let random_string: String = rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();

        UserNickname::new(random_string)
    }
}
