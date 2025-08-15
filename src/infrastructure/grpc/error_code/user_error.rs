// --- 필요한 use 구문 ---
use super::def::{Aggregate, Category, Service, StructuredErrorCode};
use bytes::Bytes;
use prost::Message;
use protos::ErrorDetail;
use tonic::{Code, Status};

use crate::application::port::inbound::{
    create_random_nickname_user_use_case::CreateRandomNicknameUserError,
    find_user_by_auth_use_case::FindUserByAuthError,
};

#[derive(Debug)]
pub enum UserError {
    AlreadyRegistered {
        auth_provider: String,
        auth_id: String,
    },
    NicknameGenerationFailed,
    CreateUserUnknown,
    FindUserUnknown,
}

impl UserError {
    fn structured_code(&self) -> StructuredErrorCode {
        match self {
            Self::AlreadyRegistered { .. } => StructuredErrorCode {
                service: Service::User,
                aggregate: Aggregate::User,
                category: Category::Business,
                number: 1,
            },
            Self::NicknameGenerationFailed | Self::CreateUserUnknown => StructuredErrorCode {
                service: Service::User,
                aggregate: Aggregate::User,
                category: Category::Server,
                number: 1,
            },
            Self::FindUserUnknown => StructuredErrorCode {
                service: Service::User,
                aggregate: Aggregate::User,
                category: Category::Server,
                number: 2,
            },
        }
    }

    pub fn to_status(self) -> Status {
        let error_code = self.structured_code().to_string();

        match self {
            Self::AlreadyRegistered {
                auth_provider,
                auth_id,
            } => {
                let detail = ErrorDetail {
                    error_code,
                    metadata: [
                        ("auth_provider".to_string(), auth_provider),
                        ("auth_id".to_string(), auth_id),
                    ]
                        .into(),
                };

                let mut buf = Vec::new();
                detail.encode(&mut buf).unwrap();
                let details_bytes = Bytes::from(buf);

                Status::with_details(
                    Code::AlreadyExists,
                    "이미 등록된 사용자입니다.",
                    details_bytes,
                )
            }
            Self::NicknameGenerationFailed | Self::CreateUserUnknown | Self::FindUserUnknown => {
                let detail = ErrorDetail {
                    error_code,
                    metadata: Default::default(),
                };
                let mut buf = Vec::new();
                detail.encode(&mut buf).unwrap();
                let details_bytes = Bytes::from(buf);

                Status::with_details(
                    Code::Internal,
                    "서버 내부 오류로 인해 요청을 처리하지 못했습니다.",
                    details_bytes,
                )
            }
        }
    }
}

impl From<CreateRandomNicknameUserError> for UserError {
    fn from(err: CreateRandomNicknameUserError) -> Self {
        match err {
            CreateRandomNicknameUserError::AlreadyRegistered(auth_provider, auth_id) => {
                UserError::AlreadyRegistered {
                    auth_provider: auth_provider.to_string(),
                    auth_id: auth_id.get_value().to_string(),
                }
            }
            CreateRandomNicknameUserError::NicknameGenerationFailed(_) => {
                UserError::NicknameGenerationFailed
            }
            CreateRandomNicknameUserError::Unknown(_) => UserError::CreateUserUnknown,
        }
    }
}

impl From<FindUserByAuthError> for UserError {
    fn from(err: FindUserByAuthError) -> Self {
        match err {
            FindUserByAuthError::Unknown(_) => UserError::FindUserUnknown,
        }
    }
}