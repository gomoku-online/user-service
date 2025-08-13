use std::fmt::Debug;
use crate::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;

pub trait UnitOfWork: Debug + Send + Sync {
    fn get_id(&self) -> &UnitOfWorkId;
}
