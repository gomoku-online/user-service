use crate::application::service::create_random_nickname_user_service::CreateRandomNicknameUserService;
use crate::application::service::find_user_by_auth_service::FindUserByAuthService;
use crate::bootstrap::app::signal::wait_shutdown_signal;
use crate::bootstrap::config::app_config::AppConfig;
use crate::bootstrap::config::oracle_config::OracleConfig;
use crate::infrastructure::generator::nickname_generator_impl::NicknameGeneratorImpl;
use crate::infrastructure::grpc::auth_interceptor::AuthInterceptor;
use crate::infrastructure::grpc::user_controller::UserGrpcController;
use crate::infrastructure::persistence::oracle::adapter::create_user_oracle_adapter::CreateUserOracleAdapter;
use crate::infrastructure::persistence::oracle::adapter::find_user_by_auth_oracle_adapter::FindUserByAuthOracleAdapter;
use crate::infrastructure::uow::oracle::oracle_uow_manager::OracleUowManager;
use anyhow::{Context, Result, anyhow};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_oci::OciConnection;
use protos::UserServiceServer;
use std::env::var;
use std::sync::Arc;
use diesel_logger::LoggingConnection;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tonic::service::InterceptorLayer;
use tonic::transport::Server;
use tracing::info;
use crate::bootstrap::migration::oracle_migration::run_oracle_migrations;
use crate::infrastructure::persistence::oracle::constraint::mapper::create_user_constraint_mapper::CreateUserConstraintMapper;
use crate::infrastructure::persistence::oracle::error_parser::OracleErrorParser;

pub async fn run_app() -> Result<()> {
    dotenvy::dotenv().ok();
    let app_config = AppConfig::load().context("Failed to load use_case config")?;

    init_tracing();

    // 1. 의존성 주입
    let oracle_conn_pool = get_oracle_connection_pool(app_config.get_oracle())
        .context("Failed to get oracle connection pool")?;

    if app_config.get_oracle().get_migration_enabled() {
        let mut conn = LoggingConnection::new(oracle_conn_pool.get()?);

        run_oracle_migrations(&mut conn).await;
    }

    // 1-1. infra 생성
    let uow_manager = Arc::new(OracleUowManager::new(oracle_conn_pool));
    let error_parser = Arc::new(OracleErrorParser);
    let user_constraint_mapper = Arc::new(CreateUserConstraintMapper);
    let create_user_oracle_adapter = Arc::new(CreateUserOracleAdapter::new(
        uow_manager.clone(),
        user_constraint_mapper,
        error_parser.clone(),
    ));
    let find_user_by_auth_oracle_adapter =
        Arc::new(FindUserByAuthOracleAdapter::new(uow_manager.clone()));
    let nickname_generator = Arc::new(NicknameGeneratorImpl);

    // 1-2. service 생성
    let create_random_nickname_user_service = Arc::new(CreateRandomNicknameUserService::new(
        uow_manager.clone(),
        Arc::new(create_user_oracle_adapter),
        nickname_generator,
    ));
    let find_user_by_auth_service = Arc::new(FindUserByAuthService::new(
        uow_manager.clone(),
        Arc::new(find_user_by_auth_oracle_adapter),
    ));

    // 1-3. Grpc 컨트롤러 생성
    let user_grpc_controller = UserGrpcController::new(
        create_random_nickname_user_service,
        find_user_by_auth_service,
    );

    // 2. 서버 실행
    let addr = format!(
        "{}:{}",
        app_config.get_server().get_host(),
        app_config.get_server().get_port()
    )
    .parse()?;

    let server_future = Server::builder()
        .layer(InterceptorLayer::new(AuthInterceptor))
        .add_service(UserServiceServer::new(user_grpc_controller))
        .serve_with_shutdown(addr, async {
            if let Err(e) = wait_shutdown_signal().await {
                tracing::error!("종료 시그널 리스너에서 오류 발생: {}", e);
            }
        });

    if let Err(e) = server_future.await {
        tracing::error!("서버 실행 중 오류 발생: {}", e);
    }

    info!("애플리케이션이 정상적으로 종료되었습니다.");
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();
}

fn get_oracle_connection_pool(
    oracle_config: &OracleConfig,
) -> anyhow::Result<Pool<ConnectionManager<OciConnection>>> {
    let database_url = oracle_config.get_url();
    let min_idle_connections = oracle_config.get_min_idle();
    let max_connections = oracle_config.get_max_conn();

    let oracle_connection_manager: ConnectionManager<OciConnection> =
        ConnectionManager::<OciConnection>::new(database_url);

    Pool::builder()
        .min_idle(Some(min_idle_connections))
        .max_size(max_connections)
        .build(oracle_connection_manager)
        .map_err(|e| anyhow!(e.to_string()))
}
