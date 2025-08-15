use crate::application::service::create_random_nickname_user_service::CreateRandomNicknameUserService;
use crate::application::service::find_user_by_auth_service::FindUserByAuthService;
use crate::infrastructure::generator::nickname_generator_impl::NicknameGeneratorImpl;
use crate::infrastructure::grpc::user_controller::UserGrpcController;
use crate::infrastructure::persistence::oracle::adapter::create_user_oracle_adapter::CreateUserOracleAdapter;
use crate::infrastructure::persistence::oracle::adapter::find_user_by_auth_oracle_adapter::FindUserByAuthOracleAdapter;
use crate::infrastructure::persistence::oracle::constraint::mapper::create_user_constraint_mapper::CreateUserConstraintMapper;
use crate::infrastructure::persistence::oracle::error_parser::OracleErrorParser;
use crate::infrastructure::uow::oracle::oracle_uow_manager::OracleUowManager;
use anyhow::Result;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_oci::OciConnection;
use getset::Getters;
use protos::UserService;
use std::sync::Arc;

#[derive(Getters)]
pub struct AppState {
    #[getset(get = "pub with_prefix")]
    user_service: Arc<UserGrpcController>,
}

impl AppState {
    pub fn new(db_pool: Pool<ConnectionManager<OciConnection>>) -> Result<Self> {
        // 1. 공통 인프라 컴포넌트 생성
        let uow_manager = Arc::new(OracleUowManager::new(db_pool));
        let error_parser = Arc::new(OracleErrorParser);
        let nickname_generator = Arc::new(NicknameGeneratorImpl);

        // 2. Persistence(Adaptor) 계층 생성
        let user_constraint_mapper = Arc::new(CreateUserConstraintMapper);
        let create_user_adapter = Arc::new(CreateUserOracleAdapter::new(
            uow_manager.clone(),
            user_constraint_mapper,
            error_parser.clone(),
        ));
        let find_user_by_auth_adapter =
            Arc::new(FindUserByAuthOracleAdapter::new(uow_manager.clone()));

        // 3. Application(Service) 계층 생성
        let create_random_nickname_user_service = Arc::new(CreateRandomNicknameUserService::new(
            uow_manager.clone(),
            Arc::new(create_user_adapter),
            nickname_generator,
        ));
        let find_user_by_auth_service = Arc::new(FindUserByAuthService::new(
            uow_manager.clone(),
            Arc::new(find_user_by_auth_adapter),
        ));

        // 4. Input Port(Controller) 계층 생성
        let user_service = Arc::new(UserGrpcController::new(
            create_random_nickname_user_service,
            find_user_by_auth_service,
        ));

        Ok(Self {
            user_service,
        })
    }
}
