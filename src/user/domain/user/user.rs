use crate::user::domain::user::user_auth::UserAuth;
use crate::user::domain::user::user_nickname::UserNickname;
use getset::{CopyGetters, Getters};
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use shared_kernel::value_object::user_id::UserId;

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
            )],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_kernel::enums::auth_provider::AuthProvider;
    use shared_kernel::value_object::auth_id::AuthId;
    use crate::user::domain::user::user_nickname::UserNickname; // 실제 경로에 맞게 수정

    #[test]
    fn create_user_should_initialize_with_correct_defaults() {
        // Given
        let nickname = UserNickname::new("test_user".to_string());
        let auth_provider = AuthProvider::KEYCLOAK;
        let auth_id = AuthId::new("some-auth-id".to_string());

        // When
        let user = User::create(nickname.clone(), auth_provider, auth_id.clone());

        // Then
        assert!(user.get_user_id().is_none(), "유저 ID는 최초 생성 시에 None으로 조회되어야합니다.");
        assert!(user.get_active(), "유저 active 필드는 최초 생성 시에 true로 조회되어야합니다.");
        assert_eq!(user.get_user_auths().len(), 1, "유저는 최초 생성 시에 user_auth가 1개여야 합니다.");
        assert_eq!(user.get_user_nickname(), &nickname);
        let user_auth = user.get_user_auths().get(0).unwrap();
        assert_eq!(user_auth.get_auth_provider(), auth_provider);
        assert_eq!(user_auth.get_auth_id(), &auth_id);
    }
}