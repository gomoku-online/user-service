use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
#[strum(serialize_all = "shouty_snake_case")]
pub enum ErrorCategory {
    InvalidRequest,
    InvalidRequestPayload,
    InternalResponseFailure,
}

#[cfg(test)]
mod tests {
    use super::ErrorCategory;
    use strum::IntoEnumIterator;

    #[test]
    fn test_all_error_domains_to_shouty_snake_case() {
        for domain in ErrorCategory::iter() {
            let actual_str = domain.to_string();

            let expected_str = match domain {
                ErrorCategory::InvalidRequest => "INVALID_REQUEST",
                ErrorCategory::InvalidRequestPayload => "INVALID_REQUEST_PAYLOAD",
                ErrorCategory::InternalResponseFailure => "INTERNAL_RESPONSE_FAILURE",
            };

            assert_eq!(
                actual_str,
                expected_str,
                "String conversion failed for variant: {:?}",
                domain
            );
        }
    }
}