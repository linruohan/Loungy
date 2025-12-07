use std::time::Duration;

use gpui::{
    App, AppContext, AsyncApp, AsyncWindowContext, BorrowAppContext, Bounds, Entity, Global,
    Pixels, Point, Size, Window, WindowBounds, WindowKind, WindowOptions,
};

use crate::{components::shared::NoView, state::StateModel, theme::Theme};

pub static WIDTH: u32 = 800;
pub static HEIGHT: u32 = 450;

pub enum LWindowStyle {
    Main,
    Toast { width: u32, height: u32 },
    Settings,
}

impl LWindowStyle {
    pub fn options(&self, bounds: Bounds<Pixels>) -> WindowOptions {
        let mut options = WindowOptions::default();
        let center = bounds.center();

        let (width, height, x, y) = match self {
            LWindowStyle::Main => {
                options.focus = true;
                let width = Pixels::from(WIDTH);
                let height = Pixels::from(HEIGHT);
                let x: Pixels = center.x - width / 2.0;
                let y: Pixels = center.y - height / 2.0;
                (width, height, x, y)
            },
            LWindowStyle::Toast { width, height } => {
                options.focus = false;
                let width = Pixels::from(*width);
                let height = Pixels::from(*height);
                let x: Pixels = center.x - width / 2.0;
                let y: Pixels = bounds.bottom() - height - Pixels::from(200.0);
                (width, height, x, y)
            },
            LWindowStyle::Settings => {
                return options;
            },
        };
        options.window_bounds =
            Some(WindowBounds::Windowed(Bounds::new(Point { x, y }, Size { width, height })));
        options.titlebar = None;
        options.is_movable = false;
        options.kind = WindowKind::PopUp;
        options
    }
}

pub struct LWindow {
    inner: Entity<NoView>,
    hidden: bool,
}

impl LWindow {
    pub fn init(window: &mut Window, cx: &mut App) {
        let view = cx.new(|cx| {
            let _ = cx
                .observe_window_activation(window, |_, window, cx| {
                    if window.is_window_active() {
                        return;
                    };
                    LWindow::close(cx);
                })
                .detach();
            cx.observe_window_appearance(window, |_, window, cx| {
                cx.update_global::<Theme, _>(|theme: &mut Theme, cx| {
                    *theme = Theme::mode(cx.window_appearance());
                    window.refresh();
                });
            })
            .detach();
            NoView {}
        });
        cx.set_global::<Self>(Self { inner: view, hidden: false });
    }

    pub fn is_open(cx: &AsyncApp) -> bool {
        cx.read_global::<Self, _>(|w, _| !w.hidden).unwrap_or(false)
    }

    pub fn open(cx: &mut App) {
        cx.update_global::<Self, _>(|this, cx| {
            if this.hidden {
                cx.activate(true);
                this.hidden = false;
            }
        });
    }

    pub fn toggle(cx: &mut App) {
        cx.update_global::<Self, _>(|this, cx| {
            if this.hidden {
                cx.activate(true);
                this.hidden = false;
            } else {
                cx.hide();
                this.hidden = true;
            }
        });
    }

    pub fn close(cx: &mut App) {
        cx.update_global::<Self, _>(|this, cx| {
            this.hidden = true;
            cx.hide();
        });
        // After 90 seconds, reset the state
        cx.spawn(async move |cx| {
            cx.background_executor().timer(Duration::from_secs(90)).await;
            let _ = cx.update_global::<Self, _>(|window, cx| {
                if window.hidden {
                    StateModel::update(|this, cx| this.reset(cx), cx);
                }
            });
        })
        .detach();
    }

    pub async fn wait_for_close(cx: &mut AsyncWindowContext) {
        while let Ok(active) = cx
            .update_window::<bool, _>(cx.window_handle(), |_, window, _| window.is_window_active())
        {
            if !active {
                break;
            }
            cx.background_executor().timer(Duration::from_millis(10)).await;
        }
    }
}

impl Global for LWindow {}
