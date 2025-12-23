/*
 *
 *  This source file is part of the Loungy open source project
 *
 *  Copyright (c) 2024 Loungy, Matthias Grandl and the Loungy project contributors
 *  Licensed under MIT License
 *
 *  See https://github.com/MatthiasGrandl/Loungy/blob/main/LICENSE.md for license information
 *
 */

use crate::components::shared::Img;
use gpui::{App, AppContext, AsyncApp, BorrowAppContext, Global};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
use std::path::PathBuf;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "windows")]
mod window;

#[cfg(target_os = "macos")]
pub use mac::*;
#[cfg(target_os = "windows")]
pub use window::*;
#[derive(Clone)]
pub struct AppData {
    pub id: String,
    pub name: String,
    pub icon: Img,
    pub icon_path: PathBuf,
    pub keywords: Vec<String>,
    pub tag: String,
}

pub struct ClipboardWatcher {
    enabled: bool,
}
impl ClipboardWatcher {
    pub fn init(cx: &mut App) {
        cx.set_global(Self { enabled: true });
    }
    pub fn enabled(cx: &mut App) {
        let _ = cx.update_global::<Self, _>(|this, _| {
            this.enabled = true;
        });
    }
    pub fn disabled(cx: &mut AsyncApp) {
        let _ = cx.update_global::<Self, _>(|this, _| {
            this.enabled = false;
        });
    }
    pub fn is_enabled(cx: &App) -> bool {
        cx.read_global::<Self, _>(|x, _| x.enabled)
    }
}
impl Global for ClipboardWatcher {}
