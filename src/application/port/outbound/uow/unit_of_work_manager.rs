use crate::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::error;

#[async_trait]
pub trait UnitOfWorkManager: Send + Sync {
    async fn begin(&self) -> Result<Arc<dyn UnitOfWork>, UnitOfWorkError>;
    async fn commit(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError>;
    async fn rollback(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError>;

    async fn execute_as_atomic<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(Arc<dyn UnitOfWork>) -> Fut + Send,
        Fut: Future<Output=Result<T, E>> + Send,
        T: Send,
        E: From<UnitOfWorkError> + Send,
    {
        let uow = self.begin().await.map_err(E::from)?;

        let result = f(uow.clone()).await;

        match result {
            Ok(value) => {
                self.commit(uow).await.map_err(E::from)?;
                Ok(value)
            }
            Err(e) => {
                if let Err(rollback_err) = self.rollback(uow).await {
                    error!(
                        "FATAL: Failed to rollback unit of work after an error_code. Rollback error_code: {}, Original error_code: {:?}",
                        rollback_err,
                        std::any::type_name::<E>()
                    );
                }
                Err(e)
            }
        }
    }
}
