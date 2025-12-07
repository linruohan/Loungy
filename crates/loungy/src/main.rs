#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use gpui::{App, AppContext, Application, Bounds, Pixels, Point, Size, WindowBackgroundAppearance};
use loungy::{
    RootCommands, Theme, Workspace,
    client::client_connect,
    run_app,
    server::{PlatformListener, setup_socket, start_server},
};
use loungy_assets::Assets;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = Application::new().with_assets(Assets);
    if let Ok(listener) = setup_socket().await {
        run_app(listener, app);
    } else if let Err(e) = client_connect().await {
        log::error!("CLI Error: {:?}", e);
    }
}
