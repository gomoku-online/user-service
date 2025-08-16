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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn new_and_get_value_should_work() {
        // Given
        let value = "test_nickname".to_string();

        // When
        let nickname = UserNickname::new(value.clone());

        // Then
        assert_eq!(nickname.get_value(), &value);
        assert_eq!(nickname.to_string(), value);
    }

    #[test]
    fn validate_should_succeed_for_valid_nickname() {
        // Given
        let nickname = UserNickname::new("valid_name".to_string());

        // When
        let result = nickname.validate();

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn validate_should_succeed_for_min_length_boundary() {
        // Given
        let nickname = UserNickname::new("a".to_string());

        // When
        let result = nickname.validate();

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn validate_should_succeed_for_max_length_boundary() {
        // Given
        let nickname = UserNickname::new("12345678901234567890".to_string());

        // When
        let result = nickname.validate();

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn validate_should_fail_when_nickname_is_too_short() {
        // Given
        let nickname = UserNickname::new("".to_string());

        // When
        let result = nickname.validate();

        // Then
        assert!(result.is_err());
        if let Err(errors) = result {
            let field_errors = errors.field_errors();

            assert!(field_errors.contains_key("value"));
            let field_errors = field_errors.get("value").unwrap();
            assert_eq!(field_errors[0].code, "length");
        }
    }

    #[test]
    fn validate_should_fail_when_nickname_is_too_long() {
        // Given
        let nickname = UserNickname::new("123456789012345678901".to_string());

        // When
        let result = nickname.validate();

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn validate_should_handle_unicode_characters_correctly() {
        // Given
        let valid_unicode_nickname = UserNickname::new("안녕".to_string());
        let invalid_unicode_nickname =
            UserNickname::new("스무글자닉네임스무글자닉네임스무글자닉네임하나".to_string());

        // When
        let valid_result = valid_unicode_nickname.validate();
        let invalid_result = invalid_unicode_nickname.validate();

        // Then
        assert!(valid_result.is_ok());
        assert!(invalid_result.is_err());
    }
}
