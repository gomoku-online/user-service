use crate::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
use crate::shared::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::error;

#[async_trait]
pub trait UnitOfWorkManager: Send + Sync {
    async fn begin(&self) -> Result<Arc<dyn UnitOfWork>, UnitOfWorkError>;
    async fn commit(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError>;
    async fn rollback(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError>;

    async fn execute_as_atomic<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(Arc<dyn UnitOfWork>) -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<UnitOfWorkError> + Send,
    {
        let uow = self.begin().await.map_err(E::from)?;

        let result = f(uow.clone()).await;

        match result {
            Ok(value) => {
                self.commit(uow).await.map_err(E::from)?;
                Ok(value)
            }
            Err(e) => {
                if let Err(rollback_err) = self.rollback(uow).await {
                    error!(
                        "UoW 롤백 실패. 롤백 오류: {}, 최초 오류 타입: {:?}",
                        rollback_err,
                        std::any::type_name::<E>()
                    );
                }
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::application::port::outbound::uow::unit_of_work::UnitOfWork;
    use crate::shared::application::port::outbound::uow::unit_of_work_error::UnitOfWorkError;
    use crate::shared::application::port::outbound::uow::unit_of_work_id::UnitOfWorkId;
    use async_trait::async_trait;
    use mockall::mock;
    use std::fmt::Debug;
    use std::future::Future;
    use std::sync::Arc;
    use thiserror::Error;

    #[derive(Debug, Error, PartialEq)]
    enum TestError {
        #[error("작업 처리 중 오류가 발생했습니다.")]
        OperationError,

        #[error("내부 서버 오류가 발생했습니다: {0}")]
        InternalServerError(#[from] UnitOfWorkError),
    }

    mock! {
        #[derive(Debug)]
        pub UnitOfWork {}

        #[async_trait]
        impl UnitOfWork for UnitOfWork {
            fn get_id(&self) -> &UnitOfWorkId;
        }
    }

    mock! {
        pub UnitOfWorkManager {}

        #[async_trait]
        impl UnitOfWorkManager for UnitOfWorkManager {
            async fn begin(&self) -> Result<Arc<dyn UnitOfWork>, UnitOfWorkError>;
            async fn commit(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError>;
            async fn rollback(&self, uow: Arc<dyn UnitOfWork>) -> Result<(), UnitOfWorkError>;
        }
    }

    #[tokio::test]
    async fn execute_as_atomic_should_commit_on_success() {
        // Given
        let mut mock_manager = MockUnitOfWorkManager::new();

        mock_manager
            .expect_begin()
            .times(1)
            .returning(|| Ok(Arc::new(MockUnitOfWork::new())));

        mock_manager
            .expect_commit()
            .times(1)
            .returning(|_uow| Ok(()));

        mock_manager.expect_rollback().times(0);

        // When
        let result: Result<i32, TestError> = mock_manager
            .execute_as_atomic(|_uow| async { Ok(100) })
            .await;

        // Then
        assert_eq!(result, Ok(100));
    }

    #[tokio::test]
    async fn execute_as_atomic_should_rollback_on_failure() {
        // Given
        let mut mock_manager = MockUnitOfWorkManager::new();

        mock_manager
            .expect_begin()
            .times(1)
            .returning(|| Ok(Arc::new(MockUnitOfWork::new())));

        mock_manager
            .expect_rollback()
            .times(1)
            .returning(|_uow| Ok(()));

        mock_manager.expect_commit().times(0);

        // When
        let result: Result<i32, TestError> = mock_manager
            .execute_as_atomic(|_uow| async { Err(TestError::OperationError) })
            .await;

        // Then
        assert_eq!(result, Err(TestError::OperationError));
    }

    #[tokio::test]
    async fn execute_as_atomic_should_return_original_error_even_if_rollback_fails() {
        // Given
        let mut mock_manager = MockUnitOfWorkManager::new();

        mock_manager
            .expect_begin()
            .times(1)
            .returning(|| Ok(Arc::new(MockUnitOfWork::new())));

        mock_manager.expect_rollback().times(1).returning(|_uow| {
            Err(UnitOfWorkError::InternalServerError(
                "롤백 DB 연결을 획득하지 못하였습니다.".to_string(),
            ))
        });

        mock_manager.expect_commit().times(0);

        // When
        let result: Result<i32, TestError> = mock_manager
            .execute_as_atomic(|_uow| async { Err(TestError::OperationError) })
            .await;

        // Then
        assert_eq!(result, Err(TestError::OperationError));
    }

    #[tokio::test]
    async fn execute_as_atomic_should_handle_begin_failure() {
        // Given
        let mut mock_manager = MockUnitOfWorkManager::new();

        mock_manager.expect_begin().times(1).returning(|| {
            Err(UnitOfWorkError::InternalServerError(
                "DB 커넥션 획득에 실패하였습니다.".to_string(),
            ))
        });

        mock_manager.expect_commit().times(0);
        mock_manager.expect_rollback().times(0);

        // When
        let result: Result<i32, TestError> = mock_manager
            .execute_as_atomic(|_uow| async { Ok(100) })
            .await;

        // Then
        match result {
            Err(TestError::InternalServerError(uow_error)) => {
                assert_eq!(
                    uow_error.to_string(),
                    "예상치 못한 내부 서버 오류가 발생했습니다: DB 커넥션 획득에 실패하였습니다."
                );
            }
            _ => panic!("내부 서버 에러를 예상했지만, 다른 에러나 OK를 받았습니다."),
        }
    }
}
