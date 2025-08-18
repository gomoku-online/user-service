use crate::user::application::port::outbound::persistence::create_user_repository::CreateUserRepositoryError;
use crate::user::infrastructure::persistence::oracle::constraint::user_constraints::UK_USERS_AUTH_PROVIDER_AUTH_ID;
use crate::user::infrastructure::persistence::oracle::constraint::user_constraints::UK_USERS_USER_NICKNAME;
use crate::shared::infrastructure::persistence::oracle::error_parser::ParsedOracleError;
use crate::shared::infrastructure::persistence::oracle::constraint_mapper::ConstraintMapper;

#[derive(Debug, Clone)]
pub struct CreateUserConstraintMapper;

impl ConstraintMapper for CreateUserConstraintMapper {
    type Error = CreateUserRepositoryError;

    fn map_error(&self, parsed_error: &ParsedOracleError) -> Option<Self::Error> {
        match parsed_error {
            ParsedOracleError::UniqueConstraintViolated(constraint_name) => {
                match *constraint_name {
                    UK_USERS_USER_NICKNAME => Some(CreateUserRepositoryError::AlreadyExistNickname),
                    UK_USERS_AUTH_PROVIDER_AUTH_ID => {
                        Some(CreateUserRepositoryError::AlreadyRegistered)
                    }
                    _ => None,
                }
            }
        }
    }
}
