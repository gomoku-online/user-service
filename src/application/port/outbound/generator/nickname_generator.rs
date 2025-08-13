use std::fmt::Debug;
use crate::domain::user::user_nickname::UserNickname;

pub trait NicknameGenerator: Debug + Send + Sync {
    fn generate(&self) -> UserNickname;
}