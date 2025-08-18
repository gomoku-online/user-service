use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_UNIQUE: Regex =
        Regex::new(r"ORA-00001: unique constraint \(\w+\.(?P<constraint>\w+)\)").unwrap();
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParsedOracleError<'a> {
    UniqueConstraintViolated(&'a str),
}

#[derive(Debug, Clone)]
pub struct OracleErrorParser;

impl OracleErrorParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse<'a>(&self, error_message: &'a str) -> Option<ParsedOracleError<'a>> {
        if let Some(caps) = RE_UNIQUE.captures(error_message) {
            return caps
                .name("constraint")
                .map(|m| ParsedOracleError::UniqueConstraintViolated(m.as_str()));
        }

        None
    }

    pub fn extract_message_from_diesel_err(
        &self,
        diesel_error: &diesel::result::Error,
    ) -> Option<String> {
        match diesel_error {
            diesel::result::Error::QueryBuilderError(boxed_err) => Some(boxed_err.to_string()),
            _ => None,
        }
    }
}
