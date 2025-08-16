use crate::test_util::oracle::oracle_connection::create_oracle_connection_pool;
use async_trait::async_trait;
use dashmap::DashMap;
use diesel::connection::TransactionManager;
use diesel::r2d2::{ConnectionManager, Pool, PoolTransactionManager, PooledConnection};
use diesel::QueryResult;
use diesel_logger::{LoggingConnection, LoggingTransactionManager};
use diesel_oci::OciConnection;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};
use tracing::error;
use user_service::application::port::outbound::uow::unit_of_work::UnitOfWork;
use user_service::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use user_service::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;
use user_service::application::port::outbound::uow::unit_of_work_manager::UnitOfWorkManager;
use user_service::infrastructure::uow::oracle::oracle_uow::OracleUow;
use user_service::infrastructure::uow::uow_sync_manager::UnitOfWorkSyncManager;

static ORACLE_DIESEL_UOW_MANAGER: OnceCell<Arc<TestOracleUowManager>> = OnceCell::const_new();

pub type ConnectionPool = Pool<ConnectionManager<OciConnection>>;
pub type Connection = LoggingConnection<PooledConnection<ConnectionManager<OciConnection>>>;

pub struct TestOracleUowManager {
    connection_pool: ConnectionPool,
    uow_connection_map: DashMap<UnitOfWorkId, Arc<Mutex<Connection>>>,
}

impl TestOracleUowManager {
    pub fn new(connection_pool: ConnectionPool) -> Self {
        Self {
            connection_pool,
            uow_connection_map: DashMap::new(),
        }
    }

    async fn finalize_transaction<F>(
        &self,
        uow_id: &UnitOfWorkId,
        db_action: F,
    ) -> Result<(), UnitOfWorkError>
    where
        F: FnOnce(&mut Connection) -> QueryResult<()> + Send + 'static,
    {
        let (_id, shared_conn) = self
            .uow_connection_map
            .remove(uow_id)
            .ok_or_else(|| UnitOfWorkError::NotFound(uow_id.clone()))?;

        let mutex_conn = Arc::try_unwrap(shared_conn)
            .map_err(|_arc_still_has_references| UnitOfWorkError::CommitConflict(uow_id.clone()))?;

        let mut conn = mutex_conn.into_inner();

        let db_result = tokio::task::spawn_blocking(move || db_action(&mut conn)).await;

        match db_result {
            Err(join_err) => Err(UnitOfWorkError::InternalServerError(join_err.to_string())),
            Ok(inner_result) => {
                inner_result.map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))
            }
        }
    }
}

#[async_trait]
impl UnitOfWorkManager for TestOracleUowManager {
    async fn begin(&self) -> Result<Arc<dyn UnitOfWork>, UnitOfWorkError> {
        let uow = OracleUow::new();
        let uow_id = uow.get_id().clone();

        let conn_result = tokio::task::spawn_blocking({
            let pool = self.connection_pool.clone();
            move || -> Result<Connection, UnitOfWorkError> {
                let mut conn = pool
                    .get()
                    .map(LoggingConnection::new)
                    .map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))?;

                LoggingTransactionManager::begin_transaction(&mut conn)
                    .map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))?;

                Ok(conn)
            }
        })
        .await
        .map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))?;

        let final_conn = conn_result?;

        self.uow_connection_map
            .insert(uow_id, Arc::new(Mutex::new(final_conn)));
        Ok(Arc::new(uow))
    }

    async fn commit(&self, transaction: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError> {
        unreachable!()
    }

    async fn rollback(&self, transaction: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError> {
        let uow_id = transaction.get_id();
        self.finalize_transaction(uow_id, |conn| {
            LoggingTransactionManager::rollback_transaction(conn)
        })
        .await
    }

    async fn execute_as_atomic<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(Arc<dyn UnitOfWork>) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<UnitOfWorkError> + Send,
    {
        let uow = self.begin().await.map_err(E::from)?;

        let result = f(uow.clone()).await;

        if let Err(rollback_err) = self.rollback(uow).await {
            error!(
                "최초 오류 발생 후 UoW(작업 단위) 롤백에 실패했습니다. 롤백 오류: {}, 최초 오류 타입: {:?}",
                rollback_err,
                std::any::type_name::<E>()
            );
        }

        result
    }
}

impl UnitOfWorkSyncManager for TestOracleUowManager {
    type Conn = Connection;
    fn get_connection_with_uow(
        &self,
        uow: Arc<dyn UnitOfWork>,
    ) -> Result<Arc<Mutex<Self::Conn>>, UnitOfWorkError> {
        let uow_id = uow.get_id();

        match self.uow_connection_map.get(uow_id) {
            Some(context_ref) => Ok(context_ref.value().clone()),
            None => Err(UnitOfWorkError::NotFound(uow_id.clone())),
        }
    }

    fn get_connection(&self) -> Result<Arc<Mutex<Self::Conn>>, UnitOfWorkError> {
        self.connection_pool
            .get()
            .map(LoggingConnection::new)
            .map(Mutex::new)
            .map(Arc::new)
            .map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))
    }
}

pub async fn get_or_create_uow_manager() -> Arc<TestOracleUowManager> {
    let uow_manager = ORACLE_DIESEL_UOW_MANAGER
        .get_or_init(|| async {
            let connection_pool = create_oracle_connection_pool().await;

            let uow_manager = TestOracleUowManager::new(connection_pool);

            Arc::new(uow_manager)
        })
        .await;

    Arc::clone(uow_manager)
}
