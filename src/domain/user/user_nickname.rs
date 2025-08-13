use std::fmt::Display;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserNickname(String);

impl UserNickname {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn get_value(&self) -> &String {
        &self.0
    }
}

impl Display for UserNickname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Validate for UserNickname {
    fn validate(&self) -> Result<(), ValidationErrors> {
        if !validator::ValidateLength::validate_length(&self.0, Some(1), Some(20), None) {
            let mut errors = ValidationErrors::new();
            let mut error = ValidationError::new("length");
            error.add_param("min".into(), &1);
            error.add_param("max".into(), &20);
            errors.add("value", error);
            return Err(errors);
        }
        Ok(())
    }
}
