use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use diesel::migration::MigrationSource;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use diesel_oci::Oracle;
use user_service::application::port::outbound::uow::unit_of_work::UnitOfWork;
use user_service::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use user_service::application::port::outbound::uow::unit_of_work_manager::UnitOfWorkManager;
use crate::test_util::oracle::log::initialize_logger;
use crate::test_util::oracle::migration::test_with_migration;
use crate::test_util::oracle::test_uow_manager::get_or_create_uow_manager;

pub async fn migrate_and_test_with_transaction<F, Fut>(migration_source: impl MigrationSource<Oracle>, f: F)
where
    F: FnOnce(Arc<dyn UnitOfWork>) -> Fut + Send,
    Fut: Future<Output = ()> + Send,
{
    initialize_logger();

    test_with_migration(migration_source, || async move {
        let tx_manager = get_or_create_uow_manager().await;

        tx_manager
            .execute_as_atomic::<_, _, (), UnitOfWorkError>(async |tx| {
                f(tx).await;

                Ok(())
            })
            .await
    })
    .await
    .expect("테스트에 실패하였습니다.");
}

pub const MIGRATION_SOURCE: EmbeddedMigrations = embed_migrations!("migrations/oracle");

pub async fn test_with_transaction<F, Fut>(f: F)
where
    F: FnOnce(Arc<dyn UnitOfWork>) -> Fut + Send,
    Fut: Future<Output=()> + Send,
{
    migrate_and_test_with_transaction(MIGRATION_SOURCE, f).await;
} 