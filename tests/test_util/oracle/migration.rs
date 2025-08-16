use crate::test_util::oracle::oracle_connection::{
    create_oracle_connection, create_oracle_system_connection,
};
use crate::test_util::oracle::oracle_test_container::USER;
use diesel::connection::SimpleConnection;
use diesel::migration::MigrationSource;
use diesel_migrations::MigrationHarness;
use diesel_oci::Oracle;
use tokio::sync::OnceCell;
use tracing::info;

static MIGRATED: OnceCell<()> = OnceCell::const_new();

pub async fn test_with_migration<F, Fut, T, E>(
    migration_source: impl MigrationSource<Oracle>,
    f: F,
) -> Result<T, E>
where
    F: FnOnce() -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
{
    MIGRATED
        .get_or_init(|| async {
            grant_privileges().await;
            migration(migration_source).await;

            ()
        })
        .await;

    f().await
}

async fn grant_privileges() {
    let mut conn = create_oracle_system_connection()
        .await
        .expect("SYSTEM(관리자) 계정으로 Oracle 데이터베이스 연결에 실패했습니다.");

    let _ =
        SimpleConnection::batch_execute(&mut conn, &format!("GRANT ALL PRIVILEGES TO {}", USER));
}

async fn migration(migration_source: impl MigrationSource<Oracle>) {
    let mut conn = create_oracle_connection()
        .await
        .expect("데이터베이스 연결에 실패했습니다.");

    info!("Oracle 마이그레이션을 시작합니다.");

    conn.run_pending_migrations(migration_source)
        .expect("데이터베이스 마이그레이션 실행에 실패했습니다.");

    info!("데이터베이스 마이그레이션이 종료되었습니다.");
}
