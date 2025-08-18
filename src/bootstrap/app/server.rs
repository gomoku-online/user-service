use crate::user::application::port::inbound::create_random_nickname_user_use_case::CreateRandomNicknameUserUseCase;
use crate::user::application::port::inbound::find_user_by_auth_use_case::FindUserByAuthQueryUseCase;
use crate::bootstrap::app::signal::wait_shutdown_signal;
use crate::bootstrap::app::state::AppState;
use crate::bootstrap::config::server_config::ServerConfig;
use crate::shared::infrastructure::grpc::auth_interceptor::AuthInterceptor;
use crate::user::infrastructure::grpc::user_controller::UserGrpcController;
use anyhow::Result;
use protos::{UserService, UserServiceServer};
use std::sync::Arc;
use tonic::service::InterceptorLayer;
use tonic::transport::Server;

pub async fn run(config: &ServerConfig, app_state: AppState) -> Result<()> {
    let addr = format!("{}:{}", config.get_host(), config.get_port()).parse()?;
    let user_service = app_state.get_user_service().clone();

    let server_future = Server::builder()
        .layer(InterceptorLayer::new(AuthInterceptor))
        .add_service(UserServiceServer::from_arc(user_service))
        .serve_with_shutdown(addr, async {
            if let Err(e) = wait_shutdown_signal().await {
                tracing::error!("종료 시그널 리스너에서 오류 발생: {}", e);
            }
        });

    tracing::info!("서버가 실행중입니다: {}", addr);

    if let Err(e) = server_future.await {
        tracing::error!("서버 실행 중 오류 발생: {}", e);
    }

    Ok(())
}
