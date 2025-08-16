use ctor::dtor;
use std::process::Command;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::sync::OnceCell;

static ORACLE_TEST_CONTAINER: OnceCell<ContainerAsync<GenericImage>> = OnceCell::const_new();
static ORACLE_DATABASE_URL: OnceCell<String> = OnceCell::const_new();
static ORACLE_DATABASE_SYSTEM_URL: OnceCell<String> = OnceCell::const_new();

pub const ORACLE_PASSWORD: &str = "test";
pub const USER: &str = "test";
pub const USER_PASSWORD: &str = "test";
pub const PORT: u16 = 1521;

async fn get_or_init_oracle_test_container() -> &'static ContainerAsync<GenericImage> {
    ORACLE_TEST_CONTAINER
        .get_or_init(|| async {
            GenericImage::new("gvenzl/oracle-free", "23-slim")
                .with_exposed_port(ContainerPort::Tcp(PORT))
                .with_wait_for(WaitFor::message_on_stdout("DATABASE IS READY TO USE!"))
                .with_network("bridge")
                .with_env_var("DEBUG", "1")
                .with_env_var("ORACLE_PASSWORD", ORACLE_PASSWORD)
                .with_env_var("APP_USER", USER)
                .with_env_var("APP_USER_PASSWORD", USER_PASSWORD)
                .start()
                .await
                .expect("오라클 컨테이너 시작을 실패하였습니다.")
        })
        .await
}

#[dtor]
fn on_shutdown() {
    if let Some(container_id) = ORACLE_TEST_CONTAINER.get().map(|c| c.id()) {
        Command::new("docker")
            .args(["container", "rm", "-f", container_id])
            .output()
            .expect("테스트 컨테이너를 멈추지 못했습니다.");
    }
}

pub async fn get_oracle_database_url() -> &'static String {
    ORACLE_DATABASE_URL
        .get_or_init(|| async {
            let port = get_or_init_oracle_test_container()
                .await
                .get_host_port_ipv4(PORT)
                .await
                .expect("오라클 데이터베이스 URL을 획득하지 못했습니다.");

            format!(
                "oracle://{}:{}@{}",
                USER,
                USER_PASSWORD,
                format!("localhost:{}/FREEPDB1", port)
            )
        })
        .await
}

pub async fn get_oracle_database_system_url() -> &'static String {
    ORACLE_DATABASE_SYSTEM_URL
        .get_or_init(|| async {
            let port = get_or_init_oracle_test_container()
                .await
                .get_host_port_ipv4(PORT)
                .await
                .expect("오라클 데이터베이스 URL을 획득하지 못했습니다.");

            format!(
                "oracle://system:{}@{}",
                ORACLE_PASSWORD,
                format!("localhost:{}/FREEPDB1", port)
            )
        })
        .await
}
