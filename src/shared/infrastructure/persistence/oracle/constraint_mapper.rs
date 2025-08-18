use crate::shared::infrastructure::persistence::oracle::error_parser::ParsedOracleError;
use std::error::Error;

pub trait ConstraintMapper {
    type Error: Error;

    fn map_error(&self, parsed_error: &ParsedOracleError) -> Option<Self::Error>;
}
