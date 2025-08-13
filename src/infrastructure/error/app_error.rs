use crate::infrastructure::error::app_status_code::AppStatusCode;
use crate::infrastructure::error::error_category::ErrorCategory;
use crate::infrastructure::error::error_domain::ErrorDomain;

pub trait AppError {
    fn get_status_code(&self) -> AppStatusCode;
    fn get_domain(&self) -> ErrorDomain;
    fn get_category(&self) -> ErrorCategory;
}