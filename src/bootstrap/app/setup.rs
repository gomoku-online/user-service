use crate::application::service::create_random_nickname_user_service::CreateRandomNicknameUserService;
use crate::application::service::find_user_by_auth_service::FindUserByAuthService;
use crate::bootstrap::app::database::create_oracle_pool;
use crate::bootstrap::app::server::run;
use crate::bootstrap::app::signal::wait_shutdown_signal;
use crate::bootstrap::app::state::AppState;
use crate::bootstrap::app::tracing::init_tracing;
use crate::bootstrap::config::app_config::AppConfig;
use crate::bootstrap::config::oracle_config::OracleConfig;
use crate::bootstrap::migration::oracle_migration::run_oracle_migrations;
use crate::infrastructure::generator::nickname_generator_impl::NicknameGeneratorImpl;
use crate::infrastructure::grpc::auth_interceptor::AuthInterceptor;
use crate::infrastructure::grpc::user_controller::UserGrpcController;
use crate::infrastructure::persistence::oracle::adapter::create_user_oracle_adapter::CreateUserOracleAdapter;
use crate::infrastructure::persistence::oracle::adapter::find_user_by_auth_oracle_adapter::FindUserByAuthOracleAdapter;
use crate::infrastructure::persistence::oracle::constraint::mapper::create_user_constraint_mapper::CreateUserConstraintMapper;
use crate::infrastructure::persistence::oracle::error_parser::OracleErrorParser;
use crate::infrastructure::uow::oracle::oracle_uow_manager::OracleUowManager;
use anyhow::{anyhow, Context, Result};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_logger::LoggingConnection;
use diesel_oci::OciConnection;
use protos::{UserService, UserServiceServer};
use std::env::var;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tonic::service::InterceptorLayer;
use tonic::transport::Server;
use tracing::info;

pub async fn run_app() -> Result<()> {
    dotenvy::dotenv().ok();
    let app_config = AppConfig::load().context("Failed to load use_case config")?;
    let oracle_config = app_config.get_oracle();
    let server_config = app_config.get_server();

    init_tracing();

    let oracle_conn_pool = create_oracle_pool(oracle_config)?;
    if oracle_config.get_migration_enabled() {
        let mut conn = LoggingConnection::new(oracle_conn_pool.get()?);

        run_oracle_migrations(&mut conn).await;
    }

    let state =
        AppState::new(oracle_conn_pool).context("애플리케이션 상태 생성에 실패하였습니다.")?;

    run(server_config, state).await?;

    info!("애플리케이션이 정상적으로 종료되었습니다.");
    Ok(())
}
