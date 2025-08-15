use super::def::{Aggregate, Category, Service, StructuredErrorCode};
use bytes::Bytes;
use prost::Message;
use protos::ErrorDetail;
use tonic::{Code, Status};

#[derive(Debug)]
pub enum AuthError {
    AuthenticationMissing,
}

impl AuthError {
    fn structured_code(&self) -> StructuredErrorCode {
        match self {
            Self::AuthenticationMissing => StructuredErrorCode {
                service: Service::User,
                aggregate: Aggregate::Auth,
                category: Category::Client,
                number: 1,
            },
        }
    }

    pub fn to_status(self) -> Status {
        let error_code = self.structured_code().to_string();

        let detail = ErrorDetail {
            error_code,
            metadata: Default::default(),
        };

        let mut buf = Vec::new();
        detail.encode(&mut buf).unwrap();
        let details_bytes = Bytes::from(buf);

        Status::with_details(
            Code::Unauthenticated,
            "요청에 유효한 사용자 인증 정보가 없습니다.",
            details_bytes,
        )
    }
}