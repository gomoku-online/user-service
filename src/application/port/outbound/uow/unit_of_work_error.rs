use crate::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnitOfWorkError {
    #[error("Unit of work not found for ID: {0}")]
    NotFound(UnitOfWorkId),

    #[error("Unit of work is already committed: {0}")]
    AlreadyCommitted(UnitOfWorkId),

    #[error("Unit of work is already rolled back: {0}")]
    AlreadyRolledBack(UnitOfWorkId),

    #[error("Attempted to commit a unit of work while its connection is still in use: {0}")]
    CommitConflict(UnitOfWorkId),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}
