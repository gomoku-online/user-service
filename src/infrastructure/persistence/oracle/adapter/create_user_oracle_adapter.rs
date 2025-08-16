use crate::application::port::outbound::persistence::create_user_repository::{
    CreateUserRepository, CreateUserRepositoryError,
};
use crate::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::domain::user::user::User;
use crate::infrastructure::persistence::oracle::constraint::mapper::constraint_mapper::ConstraintMapper;
use crate::infrastructure::persistence::oracle::error_parser::OracleErrorParser;
use crate::infrastructure::persistence::oracle::model::user::user_auth_entity::{
    NewUserAuthEntity, UserAuthEntity,
};
use crate::infrastructure::persistence::oracle::model::user::user_entity::{
    NewUserEntity, UserEntity,
};
use crate::infrastructure::persistence::oracle::schema::user::user_auths::dsl::user_auths;
use crate::infrastructure::persistence::oracle::schema::user::users::dsl::users;
use crate::infrastructure::uow::uow_sync_manager::UnitOfWorkSyncManager;
use anyhow::Error;
use async_trait::async_trait;
use diesel::connection::LoadConnection;
use diesel::dsl::insert_into;
use diesel::{Connection, RunQueryDsl};
use std::any::type_name;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub struct CreateUserOracleAdapter<M, CM> {
    uow_sync_manager: Arc<M>,
    constraint_mapper: Arc<CM>,
    error_parser: Arc<OracleErrorParser>,
}

impl<M, CM> CreateUserOracleAdapter<M, CM>
where
    M: UnitOfWorkSyncManager,
    CM: ConstraintMapper<Error=CreateUserRepositoryError>,
{
    pub fn new(
        uow_sync_manager: Arc<M>,
        constraint_mapper: Arc<CM>,
        error_parser: Arc<OracleErrorParser>,
    ) -> Self {
        Self {
            uow_sync_manager,
            constraint_mapper,
            error_parser,
        }
    }

    fn handle_db_error(&self, db_error: diesel::result::Error) -> CreateUserRepositoryError {
        if let Some(error_message) = self.error_parser.extract_message_from_diesel_err(&db_error) {
            if let Some(parsed_error) = self.error_parser.parse(&error_message) {
                if let Some(app_error) = self.constraint_mapper.map_error(&parsed_error) {
                    return app_error;
                }
            }
        }

        CreateUserRepositoryError::Unknown(db_error.to_string())
    }
}

impl<M, CM> Debug for CreateUserOracleAdapter<M, CM>
where
    M: UnitOfWorkSyncManager,
    CM: ConstraintMapper,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>()).finish()
    }
}

#[async_trait]
impl<M, CM> CreateUserRepository for Arc<CreateUserOracleAdapter<M, CM>>
where
    M: UnitOfWorkSyncManager + Send + Sync + 'static,
    M::Conn: Connection<Backend=diesel_oci::Oracle> + LoadConnection + Send + 'static,
    CM: ConstraintMapper<Error=CreateUserRepositoryError> + Send + Sync + 'static,
{
    async fn create(
        &self,
        tx: Option<Arc<dyn UnitOfWork>>,
        user: User,
    ) -> Result<User, CreateUserRepositoryError> {
        let new_user_entity = NewUserEntity {
            user_nickname: user.get_user_nickname().get_value().clone(),
            active: user.get_active(),
        };

        let new_user_auth_entities: Vec<NewUserAuthEntity> = user
            .get_user_auths()
            .iter()
            .map(|user_auth| NewUserAuthEntity {
                user_id: 0,
                auth_provider: user_auth.get_auth_provider().to_string(),
                auth_id: user_auth.get_auth_id().get_value().to_string(),
            })
            .collect();

        let conn_arc = match tx {
            None => self.uow_sync_manager.get_connection(),
            Some(tx) => self.uow_sync_manager.get_connection_with_uow(tx),
        }
            .map_err(|e| CreateUserRepositoryError::Unknown(e.to_string()))?;

        let self_clone = self.clone();

        let (created_user_entity, created_auth_entities) = tokio::task::spawn_blocking(move || {
            let mut conn_guard = conn_arc.blocking_lock();

            let result: Result<(UserEntity, Vec<UserAuthEntity>), CreateUserRepositoryError> = {
                let inserted_user = insert_into(users)
                    .values(&new_user_entity)
                    .get_result::<UserEntity>(&mut *conn_guard)
                    .map_err(|e| self_clone.handle_db_error(e))?;

                let mut auths_to_insert = new_user_auth_entities;
                for auth in &mut auths_to_insert {
                    auth.user_id = inserted_user.user_id;
                }

                let mut inserted_auths = Vec::with_capacity(auths_to_insert.len());
                for auth_entity in &auths_to_insert {
                    let inserted = insert_into(user_auths)
                        .values(auth_entity)
                        .get_result::<UserAuthEntity>(&mut *conn_guard)
                        .map_err(|e| self_clone.handle_db_error(e))?;
                    inserted_auths.push(inserted);
                }

                Ok((inserted_user, inserted_auths))
            };

            result
        })
            .await
            .map_err(|e| CreateUserRepositoryError::Unknown(format!("Task spawn failed: {}", e)))??;

        let domain_auths = created_auth_entities
            .into_iter()
            .map(|auth_entity| auth_entity.to_model())
            .collect::<Result<Vec<_>, Error>>()
            .map_err(|e| CreateUserRepositoryError::Unknown(e.to_string()))?;

        let created_user = created_user_entity.to_model(domain_auths);

        Ok(created_user)
    }
}
