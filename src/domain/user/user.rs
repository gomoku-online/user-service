use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use crate::domain::user::user_nickname::UserNickname;
use getset::{CopyGetters, Getters};
use shared_kernel::value_object::user_id::UserId;
use crate::domain::user::user_auth::UserAuth;

#[derive(CopyGetters, Getters, Debug, Clone, PartialEq, Eq)]
pub struct User {
    #[getset(get = "pub with_prefix")]
    user_id: Option<UserId>,
    #[getset(get = "pub with_prefix")]
    user_nickname: UserNickname,
    #[getset(get_copy = "pub with_prefix")]
    active: bool,
    #[getset(get = "pub with_prefix")]
    user_auths: Vec<UserAuth>,
}

impl User {
    pub fn new(
        user_id: Option<UserId>,
        user_nickname: UserNickname,
        active: bool,
        user_auths: Vec<UserAuth>,
    ) -> Self {
        Self {
            user_id,
            user_nickname,
            active,
            user_auths,
        }
    }

    pub fn create(
        user_nickname: UserNickname,
        auth_provider: AuthProvider,
        auth_id: AuthId,
    ) -> Self {
        Self {
            user_id: None,
            user_nickname,
            active: true,
            user_auths: vec![UserAuth::new(
                None,
                auth_provider,
                auth_id
            )]
        }
    }
}
