use crate::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UnitOfWorkError {
    #[error("요청하신 ID({0})에 대한 작업 단위를 찾지 못했습니다.")]
    NotFound(UnitOfWorkId),

    #[error("해당 작업 단위({0})는 이미 커밋이 완료된 상태입니다.")]
    AlreadyCommitted(UnitOfWorkId),

    #[error("해당 작업 단위({0})는 이미 롤백이 완료된 상태입니다.")]
    AlreadyRolledBack(UnitOfWorkId),

    #[error("작업 단위({0})를 커밋할 수 없습니다. 할당된 커넥션이 아직 사용 중입니다.")]
    CommitConflict(UnitOfWorkId),

    #[error("예상치 못한 내부 서버 오류가 발생했습니다: {0}")]
    InternalServerError(String),
}
