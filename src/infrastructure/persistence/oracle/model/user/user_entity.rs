use crate::domain::user::user::User;
use crate::domain::user::user_auth::UserAuth;
use crate::domain::user::user_nickname::UserNickname;
use crate::infrastructure::persistence::oracle::schema::user::users;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use shared_kernel::value_object::user_id::UserId;

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel_oci::oracle::Oracle))]
pub struct UserEntity {
    pub user_id: i64,
    pub user_nickname: String,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserEntity {
    pub fn to_model(&self, user_auths: Vec<UserAuth>) -> User {
        let user_id = Some(UserId::new(self.user_id));
        let user_nickname = UserNickname::new(self.user_nickname.clone());
        let active = self.active;

        User::new(user_id, user_nickname, active, user_auths)
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel_oci::oracle::Oracle))]
pub struct NewUserEntity {
    pub user_nickname: String,
    pub active: bool,
}
