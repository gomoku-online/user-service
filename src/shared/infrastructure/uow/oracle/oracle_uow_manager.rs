use crate::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::shared::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use crate::shared::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;
use crate::shared::application::port::outbound::uow::unit_of_work_manager::UnitOfWorkManager;
use crate::shared::infrastructure::uow::oracle::oracle_uow::OracleUow;
use crate::shared::infrastructure::uow::uow_sync_manager::UnitOfWorkSyncManager;
use async_trait::async_trait;
use dashmap::DashMap;
use diesel::connection::TransactionManager as TxManager;
use diesel::r2d2::{ConnectionManager, Pool, PoolTransactionManager, PooledConnection};
use diesel::result::QueryResult;
use diesel_logger::{LoggingConnection, LoggingTransactionManager};
use diesel_oci::OciConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type ConnectionPool = Pool<ConnectionManager<OciConnection>>;
pub type Connection = LoggingConnection<PooledConnection<ConnectionManager<OciConnection>>>;

pub struct OracleUowManager {
    connection_pool: ConnectionPool,
    uow_context_map: DashMap<UnitOfWorkId, Arc<Mutex<Connection>>>,
}

impl OracleUowManager {
    pub fn new(connection_pool: ConnectionPool) -> Self {
        Self {
            connection_pool,
            uow_context_map: DashMap::new(),
        }
    }

    async fn finalize_uow<F>(
        &self,
        uow_id: &UnitOfWorkId,
        db_action: F,
    ) -> Result<(), UnitOfWorkError>
    where
        F: FnOnce(&mut Connection) -> QueryResult<()> + Send + 'static,
    {
        let (_id, shared_conn) = self
            .uow_context_map
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

impl UnitOfWorkSyncManager for OracleUowManager {
    type Conn = Connection;

    fn get_connection_with_uow(
        &self,
        uow: Arc<dyn UnitOfWork>,
    ) -> Result<Arc<Mutex<Connection>>, UnitOfWorkError> {
        let uow_id = uow.get_id();

        match self.uow_context_map.get(uow_id) {
            Some(context_ref) => Ok(context_ref.value().clone()),
            None => Err(UnitOfWorkError::NotFound(uow_id.clone())),
        }
    }

    fn get_connection(&self) -> Result<Arc<Mutex<Connection>>, UnitOfWorkError> {
        self.connection_pool
            .get()
            .map(LoggingConnection::new)
            .map(Mutex::new)
            .map(Arc::new)
            .map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))
    }
}

#[async_trait]
impl UnitOfWorkManager for OracleUowManager {
    async fn begin(&self) -> Result<Arc<dyn UnitOfWork>, UnitOfWorkError> {
        let uow = OracleUow::new();
        let uow_id = uow.get_id().clone();

        let final_conn = tokio::task::spawn_blocking({
            let pool = self.connection_pool.clone();
            move || -> Result<Connection, UnitOfWorkError> {
                let mut conn = pool
                    .get()
                    .map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))?;

                PoolTransactionManager::begin_transaction(&mut conn)
                    .map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))?;

                Ok(LoggingConnection::new(conn))
            }
        })
            .await
            .map_err(|e| UnitOfWorkError::InternalServerError(e.to_string()))??;

        self.uow_context_map
            .insert(uow_id, Arc::new(Mutex::new(final_conn)));
        Ok(Arc::new(uow))
    }

    async fn commit(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError> {
        let uow_id = uow.get_id();
        self.finalize_uow(uow_id, |conn| {
            LoggingTransactionManager::commit_transaction(conn)
        })
            .await
    }

    async fn rollback(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError> {
        let uow_id = uow.get_id();
        self.finalize_uow(uow_id, |conn| {
            LoggingTransactionManager::rollback_transaction(conn)
        })
            .await
    }
}
