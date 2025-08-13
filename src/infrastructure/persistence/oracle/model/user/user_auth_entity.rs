use crate::domain::user::user_auth::UserAuth;
use crate::infrastructure::persistence::oracle::schema::user::user_auths;
use anyhow::Result;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use shared_kernel::value_object::user_id::UserId;
use std::str::FromStr;

#[derive(Queryable, Selectable)]
#[diesel(table_name = user_auths)]
#[diesel(check_for_backend(diesel_oci::oracle::Oracle))]
pub struct UserAuthEntity {
    pub user_auth_id: i64,
    pub user_id: i64,
    pub auth_provider: String,
    pub auth_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserAuthEntity {
    pub fn to_model(&self) -> Result<UserAuth> {
        let user_id = UserId::new(self.user_id);
        let auth_provider = AuthProvider::from_str(self.auth_provider.as_str())?;
        let auth_id = AuthId::new(self.auth_id.clone());

        Ok(UserAuth::new(Some(user_id), auth_provider, auth_id))
    }
}

#[derive(Insertable)]
#[diesel(table_name = user_auths)]
#[diesel(check_for_backend(diesel_oci::oracle::Oracle))]
pub struct NewUserAuthEntity {
    pub user_id: i64,
    pub auth_provider: String,
    pub auth_id: String,
}
