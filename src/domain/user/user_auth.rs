use getset::{CopyGetters, Getters};
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use shared_kernel::value_object::user_id::UserId;

#[derive(Getters, CopyGetters, Debug, Clone, PartialEq, Eq)]
pub struct UserAuth {
    #[getset(get_copy = "pub with_prefix")]
    user_id: Option<UserId>,
    #[getset(get_copy = "pub with_prefix")]
    auth_provider: AuthProvider,
    #[getset(get = "pub with_prefix")]
    auth_id: AuthId,
}

impl UserAuth {
    pub fn new(
        user_id: Option<UserId>,
        auth_provider: AuthProvider,
        auth_id: AuthId,
    ) -> Self {
        Self {
            user_id,
            auth_provider,
            auth_id,
        }
    }
}