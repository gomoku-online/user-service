use crate::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::shared::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;

#[derive(Debug)]
pub struct OracleUow {
    id: UnitOfWorkId,
}

impl OracleUow {
    pub fn new() -> Self {
        Self {
            id: UnitOfWorkId::new(),
        }
    }
}

impl UnitOfWork for OracleUow {
    fn get_id(&self) -> &UnitOfWorkId {
        &self.id
    }
}
