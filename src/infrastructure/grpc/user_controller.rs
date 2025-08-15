use crate::application::port::inbound::create_random_nickname_user_use_case::{
    CreateRandomNicknameUserCommand, CreateRandomNicknameUserError, CreateRandomNicknameUserUseCase,
};
use crate::application::port::inbound::find_user_by_auth_use_case::{
    FindUserByAuthError, FindUserByAuthQuery, FindUserByAuthQueryUseCase,
};
use crate::infrastructure::auth::authentication::Authentication;
use crate::infrastructure::grpc::error_code::auth_error::AuthError;
use crate::infrastructure::grpc::error_code::user_error::UserError;
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
                Err(AuthError::AuthenticationMissing.to_status())
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

        self.create_random_nickname_user_use_case
            .execute(command)
            .await
            .map(|user_summary_dto| {
                let proto_user_summary = UserSummary {
                    user_id: user_summary_dto.get_user_id().get_value(),
                    nickname: user_summary_dto.get_user_nickname().get_value().clone(),
                    active: user_summary_dto.get_active(),
                };
                Response::new(CreateRandomNicknameUserResponse {
                    user: Some(proto_user_summary),
                })
            })
            .map_err(|app_error| {
                tracing::error!("사용자 생성 유스케이스 에러 발생: {}", app_error);
                UserError::from(app_error).to_status()
            })
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

        self.find_user_by_auth_use_case
            .execute(query)
            .await
            .map(|user_summary_dto_opt| {
                let proto_user_summary_opt = user_summary_dto_opt.map(|dto| UserSummary {
                    user_id: dto.get_user_id().get_value(),
                    nickname: dto.get_user_nickname().get_value().clone(),
                    active: dto.get_active(),
                });
                Response::new(FindUserByAuthResponse {
                    user: proto_user_summary_opt,
                })
            })
            .map_err(|app_error| {
                tracing::error!("사용자 조회 유스케이스 에러 발생: {}", app_error);
                UserError::from(app_error).to_status()
            })
    }
}