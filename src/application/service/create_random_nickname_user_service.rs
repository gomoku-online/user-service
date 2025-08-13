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

#[derive(Debug, Clone)]
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
    pub fn new(uow_manager: Arc<U>, create_user_repository: Arc<P>, nickname_generator: Arc<G>) -> Self {
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
                    format!("인증 공급자: {}, 인증 ID: {}", command.get_auth_provider(), command.get_auth_id())
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
                            format!("인증 공급자: {}, 인증 ID: {}", command.get_auth_provider(), command.get_auth_id())
                        ));
                    }
                    CreateUserRepositoryError::Unknown(err_msg) => {
                        return Err(CreateRandomNicknameUserError::Unknown(err_msg));
                    }
                    CreateUserRepositoryError::AlreadyExistNickname => continue
                },
            }
        }
    }
}
