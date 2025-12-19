use std::path::PathBuf;

use crate::platform::AppData;

pub fn get_application_data(_path: &PathBuf) -> Option<AppData> {
    None
}
pub fn get_application_folders() -> Vec<std::path::PathBuf> {
    Vec::new()
}
pub fn get_application_files() -> Vec<std::path::PathBuf> {
    Vec::new()
}
pub fn get_frontmost_application_data() -> Option<AppData> {
    None
}
