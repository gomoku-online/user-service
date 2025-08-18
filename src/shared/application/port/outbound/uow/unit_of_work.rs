use crate::shared::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;
use std::fmt::Debug;

pub trait UnitOfWork: Debug + Send + Sync {
    fn get_id(&self) -> &UnitOfWorkId;
}
