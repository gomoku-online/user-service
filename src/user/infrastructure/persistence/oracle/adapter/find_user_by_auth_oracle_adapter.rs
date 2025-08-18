use crate::user::application::port::outbound::persistence::find_user_by_auth_repository::{
    FindUserByAuthRepository, FindUserByAuthRepositoryError,
};
use crate::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::user::domain::user::user::User;
use crate::user::infrastructure::persistence::oracle::model::user::user_auth_entity::UserAuthEntity;
use crate::user::infrastructure::persistence::oracle::model::user::user_entity::UserEntity;
use crate::user::infrastructure::persistence::oracle::schema::user::{user_auths, users};
use crate::shared::infrastructure::uow::uow_sync_manager::UnitOfWorkSyncManager;
use async_trait::async_trait;
use diesel::connection::LoadConnection;
use diesel::{Connection, ExpressionMethods, OptionalExtension, RunQueryDsl};
use diesel::{QueryDsl, SelectableHelper};
use shared_kernel::enums::auth_provider::AuthProvider;
use shared_kernel::value_object::auth_id::AuthId;
use std::any::type_name;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub struct FindUserByAuthOracleAdapter<M> {
    uow_sync_manager: Arc<M>,
}

impl<M> FindUserByAuthOracleAdapter<M>
where
    M: UnitOfWorkSyncManager,
{
    pub fn new(uow_sync_manager: Arc<M>) -> Self {
        Self { uow_sync_manager }
    }
}

impl<M> Debug for FindUserByAuthOracleAdapter<M>
where
    M: UnitOfWorkSyncManager,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>()).finish()
    }
}

#[async_trait]
impl<M> FindUserByAuthRepository for Arc<FindUserByAuthOracleAdapter<M>>
where
    M: UnitOfWorkSyncManager + Send + Sync + 'static,
    M::Conn: Connection<Backend=diesel_oci::Oracle> + LoadConnection + Send + 'static,
{
    async fn find(
        &self,
        uow: Option<Arc<dyn UnitOfWork>>,
        auth_provider: AuthProvider,
        auth_id: AuthId,
    ) -> Result<Option<User>, FindUserByAuthRepositoryError> {
        let conn_arc = match uow {
            None => self.uow_sync_manager.get_connection(),
            Some(tx) => self.uow_sync_manager.get_connection_with_uow(tx),
        }
            .map_err(|e| FindUserByAuthRepositoryError::Unknown(e.to_string()))?;

        let user_opt = tokio::task::spawn_blocking(move || {
            let mut conn_guard = conn_arc.blocking_lock();

            let result: Result<Option<User>, FindUserByAuthRepositoryError> = {
                let target_user_id = user_auths::table
                    .filter(user_auths::auth_provider.eq(auth_provider.to_string()))
                    .filter(user_auths::auth_id.eq(auth_id.to_string()))
                    .select(user_auths::user_id)
                    .first::<i64>(&mut *conn_guard)
                    .optional()
                    .map_err(|e| FindUserByAuthRepositoryError::Unknown(e.to_string()))?;

                let Some(user_id) = target_user_id else {
                    return Ok(None);
                };

                let user_entity_opt = users::table
                    .filter(users::user_id.eq(user_id))
                    .select(UserEntity::as_select())
                    .first::<UserEntity>(&mut *conn_guard)
                    .optional()
                    .map_err(|e| FindUserByAuthRepositoryError::Unknown(e.to_string()))?;

                let user_auth_entities = user_auths::table
                    .filter(user_auths::user_id.eq(user_id))
                    .select(UserAuthEntity::as_select())
                    .load::<UserAuthEntity>(&mut *conn_guard)
                    .map_err(|e| FindUserByAuthRepositoryError::Unknown(e.to_string()))?;

                let user_auths = user_auth_entities
                    .into_iter()
                    .map(|auth| auth.to_model())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| FindUserByAuthRepositoryError::Unknown(e.to_string()))?;

                let user = user_entity_opt.map(|user_entity| user_entity.to_model(user_auths));

                Ok(user)
            };

            result
        })
            .await
            .map_err(|e| FindUserByAuthRepositoryError::Unknown(e.to_string()))??;

        Ok(user_opt)
    }
}
