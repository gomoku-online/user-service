use crate::domain::user::user_nickname::UserNickname;
use getset::{CopyGetters, Getters};
use shared_kernel::value_object::user_id::UserId;

#[derive(Debug, Getters, CopyGetters)]
pub struct UserSummaryDto {
    #[getset(get_copy = "pub with_prefix")]
    user_id: UserId,
    #[getset(get = "pub with_prefix")]
    user_nickname: UserNickname,
    #[getset(get_copy = "pub with_prefix")]
    active: bool,
}

impl UserSummaryDto {
    pub fn new(user_id: UserId, user_nickname: UserNickname, active: bool) -> Self {
        Self {
            user_id,
            user_nickname,
            active,
        }
    }
}
