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

use crate::{
    assets::Assets, commands::RootCommands, hotkey::HotkeyManager, ipc::server::start_server,
    theme::Theme, window::Window, workspace::Workspace,
};
use async_std::os::unix::net::UnixListener;
use gpui::*;
use gpui_component::TitleBar;

pub fn run_app(listener: UnixListener, app: gpui::Application) {
    app.with_assets(Assets).run(move |cx: &mut App| {
        Theme::init(cx);
        // TODO: This still only works for a single display
        let bounds = Bounds {
            origin: Point::new(px::from(0.0), px::from(0.0)),
            size: Size {
                width: px::from(1920.0),
                height: px::from(1080.0),
            },
        };
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            window_min_size: Some(gpui::Size {
                width: px(480.),
                height: px(320.),
            }),
            kind: WindowKind::Normal,
            #[cfg(target_os = "linux")]
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            #[cfg(target_os = "linux")]
            window_decorations: Some(gpui::WindowDecorations::Client),
            #[cfg(target_os = "windows")]
            window_decorations: Some(gpui::WindowDecorations::Client),
            ..Default::default()
        };

        let _ = cx.open_window(options, |window, cx| {
            let theme = cx.global::<Theme>();
            RootCommands::init(cx);
            cx.spawn(async move |cx| start_server(listener, cx))
                .detach();
            HotkeyManager::init(cx);
            let view = Workspace::build(cx);
            Window::init(cx);
            view
        });
    });
}
