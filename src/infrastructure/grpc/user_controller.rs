use crate::application::port::inbound::create_random_nickname_user_use_case::{
    CreateRandomNicknameUserCommand, CreateRandomNicknameUserError, CreateRandomNicknameUserUseCase,
};
use crate::application::port::inbound::find_user_by_auth_use_case::{
    FindUserByAuthError, FindUserByAuthQuery, FindUserByAuthQueryUseCase,
};
use crate::infrastructure::auth::authentication::Authentication;
use async_trait::async_trait;
use protos::{
    CreateRandomNicknameUserRequest, CreateRandomNicknameUserResponse, FindUserByAuthRequest,
    FindUserByAuthResponse, UserService, UserSummary,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct UserGrpcController {
    create_random_nickname_user_use_case: Arc<dyn CreateRandomNicknameUserUseCase>,
    find_user_by_auth_use_case: Arc<dyn FindUserByAuthQueryUseCase>,
}

impl UserGrpcController {
    pub fn new(create_random_nickname_user_use_case: Arc<dyn CreateRandomNicknameUserUseCase>, find_user_by_auth_use_case: Arc<dyn FindUserByAuthQueryUseCase>) -> Self {
        Self {
            create_random_nickname_user_use_case,
            find_user_by_auth_use_case,
        }
    }

    fn extract_authentication<R>(&self, request: &Request<R>) -> Result<Authentication, Status> {
        match request.extensions().get::<Authentication>() {
            Some(auth) => Ok(auth.clone()),
            None => {
                tracing::error!(
                    "인증 인터셉터가 정상 동작하지 않음: 요청에서 인증 정보를 찾을 수 없습니다."
                );

                Err(Status::internal(
                    "서버 내부 오류: 사용자 인증 정보를 처리하지 못했습니다.",
                ))
            }
        }
    }
}

#[async_trait]
impl UserService for UserGrpcController {
    async fn create_random_nickname_user(
        &self,
        request: Request<CreateRandomNicknameUserRequest>,
    ) -> Result<Response<CreateRandomNicknameUserResponse>, Status> {
        let authentication = self.extract_authentication(&request)?;

        let command = CreateRandomNicknameUserCommand::new(
            authentication.get_auth_provider(),
            authentication.get_auth_id().clone(),
        );

        match self.create_random_nickname_user_use_case.execute(command).await {
            Ok(user_summary_dto) => {
                let proto_user_summary = UserSummary {
                    user_id: user_summary_dto.get_user_id().get_value(),
                    nickname: user_summary_dto.get_user_nickname().get_value().clone(),
                    active: user_summary_dto.get_active(),
                }
                    .into();

                let response = CreateRandomNicknameUserResponse {
                    user: Some(proto_user_summary),
                };
                Ok(Response::new(response))
            }
            Err(error) => Err(map_create_user_error_to_status(error)),
        }
    }

    async fn find_user_by_auth(
        &self,
        request: Request<FindUserByAuthRequest>,
    ) -> Result<Response<FindUserByAuthResponse>, Status> {
        let authentication = self.extract_authentication(&request)?;

        let query = FindUserByAuthQuery::new(
            authentication.get_auth_provider(),
            authentication.get_auth_id().clone(),
        );

        match self.find_user_by_auth_use_case.execute(query).await {
            Ok(user_summary_dto_opt) => {
                let proto_user_summary_opt = user_summary_dto_opt.map(|dto| UserSummary {
                    user_id: dto.get_user_id().get_value(),
                    nickname: dto.get_user_nickname().get_value().clone(),
                    active: dto.get_active(),
                });

                let response = FindUserByAuthResponse {
                    user: proto_user_summary_opt,
                };
                Ok(Response::new(response))
            }
            Err(error) => Err(map_find_user_error_to_status(error)),
        }
    }
}

fn map_create_user_error_to_status(error: CreateRandomNicknameUserError) -> Status {
    tracing::error!("사용자 생성 유스케이스 에러 발생: {}", error);

    match error {
        CreateRandomNicknameUserError::AlreadyRegistered(_) => {
            Status::already_exists("이미 등록된 사용자입니다.")
        }

        CreateRandomNicknameUserError::NicknameGenerationFailed(_)
        | CreateRandomNicknameUserError::Unknown(_) => {
            Status::internal("서버 내부 오류로 인해 요청을 처리하지 못했습니다.")
        }
    }
}

fn map_find_user_error_to_status(error: FindUserByAuthError) -> Status {
    tracing::error!("사용자 조회 유스케이스 에러 발생: {}", error);

    match error {
        FindUserByAuthError::Unknown(_) => {
            Status::internal("서버 내부 오류로 인해 요청을 처리하지 못했습니다.")
        }
    }
}
