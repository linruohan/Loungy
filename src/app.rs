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

use crate::ipc::server::PlatformListener;
use crate::{
    assets::Assets, commands::RootCommands, hotkey::HotkeyManager, ipc::server::start_server,
    theme::Theme, window::LWindow, workspace::Workspace,
};
use gpui::*;
use gpui_component::TitleBar;

pub fn run_app(listener: PlatformListener, app: gpui::Application) {
    app.with_assets(Assets).run(move |cx: &mut App| {
        Theme::init(cx);
        // TODO: This still only works for a single display
        let bounds = cx.displays().first().map(|d| d.bounds()).unwrap_or(Bounds {
            origin: Point::new(Pixels::from(0.0), Pixels::from(0.0)),
            size: Size {
                width: Pixels::from(1920.0),
                height: Pixels::from(1080.0),
            },
        });
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

        let _ = cx.open_window(options, |cx| {
            let theme = cx.global::<Theme>();
            cx.set_background_appearance(WindowBackgroundAppearance::from(
                theme.window_background.clone().unwrap_or_default(),
            ));
            RootCommands::init(cx);
            cx.spawn(|cx| start_server(listener, cx), ()).detach();
            HotkeyManager::init(cx);
            let view = Workspace::build(cx);
            LWindow::init(cx);
            view
        });
    });
}
