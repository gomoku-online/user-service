use crate::application::dtos::user_dto::UserSummaryDto;
use crate::application::port::inbound::create_random_nickname_user_use_case::{
    CreateRandomNicknameUserCommand, CreateRandomNicknameUserError, CreateRandomNicknameUserUseCase,
};
use crate::application::port::outbound::generator::nickname_generator::NicknameGenerator;
use crate::application::port::outbound::persistence::create_user_repository::{
    CreateUserRepository, CreateUserRepositoryError,
};
use crate::application::port::outbound::uow::unit_of_work_manager::UnitOfWorkManager;
use crate::domain::user::user::User;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct CreateRandomNicknameUserService<U, P, G>
where
    U: UnitOfWorkManager + ?Sized,
    P: CreateUserRepository + ?Sized,
    G: NicknameGenerator + ?Sized,
{
    uow_manager: Arc<U>,
    create_user_repository: Arc<P>,
    nickname_generator: Arc<G>,
}

impl<U, P, G> CreateRandomNicknameUserService<U, P, G>
where
    U: UnitOfWorkManager + ?Sized,
    P: CreateUserRepository + ?Sized,
    G: NicknameGenerator + ?Sized,
{
    pub fn new(
        uow_manager: Arc<U>,
        create_user_repository: Arc<P>,
        nickname_generator: Arc<G>,
    ) -> Self {
        Self {
            uow_manager,
            create_user_repository,
            nickname_generator,
        }
    }
}

#[async_trait]
impl<U, P, G> CreateRandomNicknameUserUseCase for CreateRandomNicknameUserService<U, P, G>
where
    U: UnitOfWorkManager + ?Sized,
    P: CreateUserRepository + ?Sized,
    G: NicknameGenerator + ?Sized,
{
    async fn execute(
        &self,
        command: CreateRandomNicknameUserCommand,
    ) -> Result<UserSummaryDto, CreateRandomNicknameUserError> {
        const MAX_ATTEMPTS: u32 = 5;
        let mut attempts = 0;

        loop {
            if attempts >= MAX_ATTEMPTS {
                return Err(CreateRandomNicknameUserError::NicknameGenerationFailed(
                    format!(
                        "인증 공급자: {}, 인증 ID: {}",
                        command.get_auth_provider(),
                        command.get_auth_id()
                    ),
                ));
            }
            attempts += 1;

            let nickname = self.nickname_generator.generate();

            let new_user = User::create(
                nickname,
                command.get_auth_provider(),
                command.get_auth_id().clone(),
            );

            let result = self
                .uow_manager
                .execute_as_atomic(|tx| async move {
                    self.create_user_repository.create(Some(tx), new_user).await
                })
                .await;

            match result {
                Ok(saved_user) => {
                    let user_id = saved_user.get_user_id().ok_or_else(|| {
                        CreateRandomNicknameUserError::Unknown(
                            "유저 구조체에 유저 ID가 존재하지 않습니다.".to_string(),
                        )
                    })?;

                    let user_nickname = saved_user.get_user_nickname().clone();
                    let active = saved_user.get_active();

                    return Ok(UserSummaryDto::new(user_id, user_nickname, active));
                }
                Err(e) => match e {
                    CreateUserRepositoryError::AlreadyRegistered => {
                        return Err(CreateRandomNicknameUserError::AlreadyRegistered(
                            command.get_auth_provider(),
                            command.get_auth_id().clone(),
                        ));
                    }
                    CreateUserRepositoryError::Unknown(err_msg) => {
                        return Err(CreateRandomNicknameUserError::Unknown(err_msg));
                    }
                    CreateUserRepositoryError::AlreadyExistNickname => continue,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::port::inbound::create_random_nickname_user_use_case::CreateRandomNicknameUserCommand;
    use crate::application::port::outbound::generator::nickname_generator::NicknameGenerator;
    use crate::application::port::outbound::persistence::create_user_repository::{
        CreateUserRepository, CreateUserRepositoryError,
    };
    use crate::application::port::outbound::uow::unit_of_work::UnitOfWork;
    use crate::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
    use crate::application::port::outbound::uow::unit_of_work_manager::UnitOfWorkManager;
    use crate::domain::user::user::User;
    use crate::domain::user::user_nickname::UserNickname;
    use async_trait::async_trait;
    use mockall::{mock, Sequence};
    use shared_kernel::enums::auth_provider::AuthProvider;
    use shared_kernel::value_object::auth_id::AuthId;
    use shared_kernel::value_object::user_id::UserId;
    use std::future::Future;
    use std::sync::Arc;
    use crate::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;

    mock! {
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
        pub CreateUserRepository {}
        #[async_trait]
        impl CreateUserRepository for CreateUserRepository {
             async fn create(
                &self,
                uow: Option<Arc<dyn UnitOfWork>>,
                user: User,
            ) -> Result<User, CreateUserRepositoryError>;
        }
    }

    mock! {
        #[derive(Debug)]
        pub NicknameGenerator {}
        impl NicknameGenerator for NicknameGenerator {
            fn generate(&self) -> UserNickname;
        }
    }

    mock! {
        #[derive(Debug)]
        pub UnitOfWork {}
        #[async_trait]
        impl UnitOfWork for UnitOfWork {
            fn get_id(&self) -> &UnitOfWorkId;
        }
    }

    struct Mocks {
        uow_manager: MockUnitOfWorkManager,
        repo: MockCreateUserRepository,
        generator: MockNicknameGenerator,
    }

    fn setup() -> Mocks {
        Mocks {
            uow_manager: MockUnitOfWorkManager::new(),
            repo: MockCreateUserRepository::new(),
            generator: MockNicknameGenerator::new(),
        }
    }

    fn expect_successful_creation(
        mocks: &mut Mocks,
        sequence: &mut Sequence,
        nickname: UserNickname,
        expected_user_id: UserId,
    ) {
        mocks
            .generator
            .expect_generate()
            .times(1)
            .in_sequence(sequence)
            .return_const(nickname);
        mocks
            .uow_manager
            .expect_begin()
            .times(1)
            .in_sequence(sequence)
            .returning(|| Ok(Arc::new(MockUnitOfWork::new())));
        mocks
            .repo
            .expect_create()
            .times(1)
            .in_sequence(sequence)
            .returning(move |_tx, user| {
                Ok(User::new(
                    Some(expected_user_id),
                    user.get_user_nickname().clone(),
                    true,
                    vec![],
                ))
            });
        mocks
            .uow_manager
            .expect_commit()
            .times(1)
            .in_sequence(sequence)
            .returning(|_uow| Ok(()));
    }

    fn expect_failed_creation(
        mocks: &mut Mocks,
        sequence: &mut Sequence,
        nickname: UserNickname,
        error: CreateUserRepositoryError,
    ) {
        mocks
            .generator
            .expect_generate()
            .times(1)
            .in_sequence(sequence)
            .return_const(nickname);
        mocks
            .uow_manager
            .expect_begin()
            .times(1)
            .in_sequence(sequence)
            .returning(|| Ok(Arc::new(MockUnitOfWork::new())));
        mocks
            .repo
            .expect_create()
            .times(1)
            .in_sequence(sequence)
            .returning(move |_tx, _user| Err(error.clone()));
        mocks
            .uow_manager
            .expect_rollback()
            .times(1)
            .in_sequence(sequence)
            .returning(|_uow| Ok(()));
    }

    #[tokio::test]
    async fn success_on_first_try() {
        // Given
        let mut mocks = setup();
        let command = CreateRandomNicknameUserCommand::new(
            AuthProvider::KEYCLOAK,
            AuthId::new("test-auth-id".to_string()),
        );
        let nickname = UserNickname::new("cool_nickname".to_string());
        let expected_user_id = UserId::new(1);
        let mut sequence = Sequence::new();

        expect_successful_creation(
            &mut mocks,
            &mut sequence,
            nickname.clone(),
            expected_user_id,
        );
        mocks.uow_manager.expect_rollback().times(0);

        let service = CreateRandomNicknameUserService::new(
            Arc::new(mocks.uow_manager),
            Arc::new(mocks.repo),
            Arc::new(mocks.generator),
        );

        // When
        let result = service.execute(command).await;

        // Then
        assert!(result.is_ok());
        let user_dto = result.unwrap();
        assert_eq!(user_dto.get_user_id(), expected_user_id);
        assert_eq!(user_dto.get_user_nickname().get_value(), "cool_nickname");
    }

    #[tokio::test]
    async fn success_after_one_retry_on_nickname_conflict() {
        // Given
        let mut mocks = setup();
        let command = CreateRandomNicknameUserCommand::new(
            AuthProvider::KEYCLOAK,
            AuthId::new("test-auth-id".to_string()),
        );
        let nickname1 = UserNickname::new("exist_nickname".to_string());
        let nickname2 = UserNickname::new("new_nickname".to_string());
        let expected_user_id = UserId::new(1);
        let mut sequence = Sequence::new();

        expect_failed_creation(
            &mut mocks,
            &mut sequence,
            nickname1,
            CreateUserRepositoryError::AlreadyExistNickname,
        );
        expect_successful_creation(
            &mut mocks,
            &mut sequence,
            nickname2.clone(),
            expected_user_id,
        );

        let service = CreateRandomNicknameUserService::new(
            Arc::new(mocks.uow_manager),
            Arc::new(mocks.repo),
            Arc::new(mocks.generator),
        );

        // When
        let result = service.execute(command).await;

        // Then
        assert!(result.is_ok());
        let user_dto = result.unwrap();
        assert_eq!(user_dto.get_user_nickname().get_value(), "new_nickname");
    }

    #[tokio::test]
    async fn fail_after_max_retries() {
        // Given
        let mut mocks = setup();
        let command = CreateRandomNicknameUserCommand::new(
            AuthProvider::KEYCLOAK,
            AuthId::new("test-auth-id".to_string()),
        );
        let mut sequence = Sequence::new();

        for _ in 0..5 {
            let nickname = UserNickname::new("any_nickname".to_string());
            expect_failed_creation(
                &mut mocks,
                &mut sequence,
                nickname,
                CreateUserRepositoryError::AlreadyExistNickname,
            );
        }

        mocks.uow_manager.expect_commit().times(0);
        let service = CreateRandomNicknameUserService::new(
            Arc::new(mocks.uow_manager),
            Arc::new(mocks.repo),
            Arc::new(mocks.generator),
        );

        // When
        let result = service.execute(command).await;

        // Then
        assert!(result.is_err());
        match result.err().unwrap() {
            CreateRandomNicknameUserError::NicknameGenerationFailed(_) => (),
            other_error => panic!(
                "예상된 오류는 'NicknameGenerationFailed'이지만, '{:?}' 오류가 발생했습니다.",
                other_error
            ),
        }
    }

    #[tokio::test]
    async fn fail_when_user_already_registered() {
        // Given
        let mut mocks = setup();
        let command = CreateRandomNicknameUserCommand::new(
            AuthProvider::KEYCLOAK,
            AuthId::new("registered-auth-id".to_string()),
        );
        let nickname = UserNickname::new("any_nickname".to_string());
        let mut sequence = Sequence::new();

        expect_failed_creation(
            &mut mocks,
            &mut sequence,
            nickname,
            CreateUserRepositoryError::AlreadyRegistered,
        );

        mocks.uow_manager.expect_commit().times(0);
        let service = CreateRandomNicknameUserService::new(
            Arc::new(mocks.uow_manager),
            Arc::new(mocks.repo),
            Arc::new(mocks.generator),
        );

        // When
        let result = service.execute(command).await;

        // Then
        assert!(result.is_err());
        match result.err().unwrap() {
            CreateRandomNicknameUserError::AlreadyRegistered(_, _) => (),
            other_error => panic!(
                "예상된 오류는 'AlreadyRegistered'이지만, '{:?}' 오류가 발생했습니다.",
                other_error
            ),
        }
    }
}
