// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adb_commands;
mod instance;
mod logging;
mod server;
mod ui;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use gpui_kit::*;
use instance::Startup;

fn main() {
    logging::init();

    let guard = match instance::claim() {
        Startup::Primary(guard) => guard,
        Startup::AlreadyRunning => return,
    };

    let (focus_tx, mut focus_rx) = tokio::sync::mpsc::unbounded_channel();
    guard.listen_for_focus(focus_tx);
    // Hold the instance lock for the whole process; the OS releases it on exit.
    std::mem::forget(guard);

    // The local HTTP server (axum) needs tokio; GPUI owns the main thread.
    let server_running = Arc::new(AtomicBool::new(false));
    let port_taken = match server::bind() {
        Ok(listener) => {
            let running = server_running.clone();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build tokio runtime");
                runtime.block_on(server::launch_server(listener, running));
            });
            false
        }
        Err(err) => {
            tracing::error!("port {} is unavailable: {err}", server::PORT);
            true
        }
    };

    gpui_kit::application().run(move |cx: &mut App| {
        gpui_kit::init(cx);

        // Closing the window quits the app (and the local server with it).
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(400.), px(400.)), cx);
        let (window, _) = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Kodular Starter".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|cx| ui::StarterView::new(server_running, port_taken, cx)),
        )
        .expect("failed to open window");
        cx.activate(true);

        // A second launch asked us to come to the front.
        cx.spawn(async move |cx| {
            while focus_rx.recv().await.is_some() {
                let _ = window.update(cx, |_, window, _| window.activate_window());
                cx.update(|cx| cx.activate(true));
            }
        })
        .detach();
    });
}
