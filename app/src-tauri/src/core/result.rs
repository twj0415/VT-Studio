use crate::core::error::AppErrorDto;

pub type AppResult<T> = Result<T, AppErrorDto>;
