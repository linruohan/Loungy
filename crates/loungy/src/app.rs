use crate::{
    RootCommands, ipc::server::PlatformListener, theme::Theme, window::LWindow,
    workspace::Workspace,
};
use gpui::{
    App, Bounds, Pixels, Point, Size, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions, px,
};
use gpui_component::TitleBar;

pub fn run_app(listener: PlatformListener, app: gpui::Application) {
    app.run(move |cx: &mut App| {
        Theme::init(cx);
        RootCommands::init(cx);
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
            window_min_size: Some(Size {
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
            window.set_background_appearance(WindowBackgroundAppearance::from(
                theme.window_background.clone().unwrap_or_default(),
            ));
            // RootCommands::init(cx);
            // let _ = cx.spawn(async move |cx| start_server(listener, cx));
            // HotkeyManager::init(cx);
            let view = Workspace::build(window, cx);
            LWindow::init(window, cx);
            view
        });
    });
}
