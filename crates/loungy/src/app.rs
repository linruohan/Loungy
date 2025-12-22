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
    commands::RootCommands,
    hotkey::HotkeyManager,
    ipc::server::start_server,
    theme::LTheme,
    window::{LWindow, LWindowStyle},
    workspace::Workspace,
};
use gpui::{App, Bounds, Pixels, Point, Size, WindowBackgroundAppearance};
use loungy_assets::Assets;
pub fn run_app(listener: PlatformListener, app: gpui::Application) {
    app.with_assets(Assets).run(move |cx: &mut App| {
        LTheme::init(cx);
        // TODO: This still only works for a single display
        let bounds = cx.displays().first().map(|d| d.bounds()).unwrap_or(Bounds {
            origin: Point::new(Pixels::from(0.0), Pixels::from(0.0)),
            size: Size {
                width: Pixels::from(1920.0),
                height: Pixels::from(1080.0),
            },
        });
        let _ = cx.open_window(LWindowStyle::Main.options(bounds), |window, cx| {
            let theme = cx.global::<LTheme>();
            cx.set_background_appearance(WindowBackgroundAppearance::from(
                theme.window_background.clone().unwrap_or_default(),
            ));
            RootCommands::init(cx);
            cx.spawn(|cx| start_server(listener, cx)).detach();
            HotkeyManager::init(cx);
            let view = Workspace::build(window, cx);
            LWindow::init(window, cx);

            view
        });
    });
}
