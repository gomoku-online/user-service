use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum Service {
    User,
}

#[derive(Debug, Clone, Copy)]
pub enum Aggregate {
    User,
    Auth,
}

#[derive(Debug, Clone, Copy)]
pub enum Category {
    Client,
    Business,
    Server,
}

impl Display for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "USR"),
        }
    }
}

impl Display for Aggregate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "USER"),
            Self::Auth => write!(f, "AUTH"),
        }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Client => write!(f, "CLIENT"),
            Self::Business => write!(f, "BIZ"),
            Self::Server => write!(f, "SERVER"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StructuredErrorCode {
    pub service: Service,
    pub aggregate: Aggregate,
    pub category: Category,
    pub number: u16,
}

impl Display for StructuredErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}-{:03}",
            self.service, self.aggregate, self.category, self.number
        )
    }
}