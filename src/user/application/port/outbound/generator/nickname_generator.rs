use crate::user::domain::user::user_nickname::UserNickname;
use std::fmt::Debug;

pub trait NicknameGenerator: Debug + Send + Sync {
    fn generate(&self) -> UserNickname;
}