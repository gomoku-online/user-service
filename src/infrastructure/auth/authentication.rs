use getset::{CopyGetters, Getters};
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;

#[derive(Debug, Clone, CopyGetters, Getters)]
pub struct Authentication {
    #[getset(get_copy = "pub with_prefix")]
    auth_provider: AuthProvider,
    #[getset(get = "pub with_prefix")]
    auth_id: AuthId
}

impl Authentication {
    pub fn new(
        auth_provider: AuthProvider,
        auth_id: AuthId
    ) -> Self {
        Self {
            auth_provider,
            auth_id
        }
    }
}