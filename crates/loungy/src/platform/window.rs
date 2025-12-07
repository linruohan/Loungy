//  This source file is part of the Loungy open source project
//
//  Copyright (c) 2024 Loungy, Matthias Grandl and the Loungy project contributors
//  Licensed under MIT License
//
//  See https://github.com/MatthiasGrandl/Loungy/blob/main/LICENSE.md for license information
//
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
