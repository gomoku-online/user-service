use crate::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::shared::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use diesel::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub trait UnitOfWorkSyncManager: Send + Sync {
    type Conn: Connection;
    /// 작업의 단위에 할당된 커넥션을 반환합니다.
    /// 작업의 단위가 시작된 경우 - Ok 반환
    /// 작업의 단위가 commit, rollback 된 경우 - Err 반환
    fn get_connection_with_uow(
        &self,
        uow: Arc<dyn UnitOfWork>,
    ) -> Result<Arc<Mutex<Self::Conn>>, UnitOfWorkError>;
    /// 커넥션 풀에서 커넥션을 반환
    fn get_connection(&self) -> Result<Arc<Mutex<Self::Conn>>, UnitOfWorkError>;
}
