use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
#[strum(serialize_all = "shouty_snake_case")]
pub enum ErrorDomain {
    Request,
    Response,
}

#[cfg(test)]
mod tests {
    use super::ErrorDomain;
    use strum::IntoEnumIterator;

    #[test]
    fn test_all_error_domains_to_shouty_snake_case() {
        for domain in ErrorDomain::iter() {
            let actual_str = domain.to_string();

            let expected_str = match domain {
                ErrorDomain::Request => "REQUEST",
                ErrorDomain::Response => "RESPONSE",
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

