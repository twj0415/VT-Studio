use crate::core::error::AppErrorDto;
use crate::core::result::AppResult;
use crate::domain::dictionary::DictionaryDto;
use crate::services::dictionary_service;

#[tauri::command]
pub fn list_dictionaries() -> AppResult<Vec<DictionaryDto>> {
    dictionary_service::list_dictionaries().map_err(AppErrorDto::from)
}

#[tauri::command]
pub fn get_dictionary(code: String) -> AppResult<DictionaryDto> {
    dictionary_service::get_dictionary(code).map_err(AppErrorDto::from)
}
