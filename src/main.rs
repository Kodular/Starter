// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adb_commands;
mod server;
mod ui;

use gpui_kit::*;

fn main() {
    // The local HTTP server (axum) needs tokio; GPUI owns the main thread.
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        runtime.block_on(server::launch_server());
    });

    gpui_kit::application().run(|cx: &mut App| {
        gpui_kit::init(cx);

        // Closing the window quits the app (and the local server with it).
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(400.), px(400.)), cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Kodular Starter".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(ui::StarterView::new),
        )
        .expect("failed to open window");
        cx.activate(true);
    });
}
