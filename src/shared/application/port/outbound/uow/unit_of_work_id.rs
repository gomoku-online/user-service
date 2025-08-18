use std::borrow::Borrow;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitOfWorkId(Uuid);

impl UnitOfWorkId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for UnitOfWorkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
