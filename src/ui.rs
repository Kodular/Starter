use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::component::description_list::DescriptionList;
use gpui_kit::component::group_box::GroupBox;
use gpui_kit::component::link::Link;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::adb_commands::DeviceInfo;
use crate::device::DeviceHandle;

/// How often the window re-reads the device snapshot and server flag (cheap: no USB).
const REFRESH_INTERVAL: Duration = Duration::from_millis(500);
const BRAND_COLOR: u32 = 0x4629a0;
const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

pub struct StarterView {
    /// Set by the server thread while it is serving.
    server_flag: Arc<AtomicBool>,
    /// Port 8004 is held by a program other than Kodular Starter.
    port_taken: bool,
    server_running: bool,
    device: Option<DeviceInfo>,
    logo: Arc<Image>,
    _poll: Task<()>,
}

impl StarterView {
    pub fn new(
        device: DeviceHandle,
        server_flag: Arc<AtomicBool>,
        port_taken: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let poll = cx.spawn(async move |this, cx| {
            loop {
                let latest = device.current();
                let alive = this.update(cx, |this, cx| {
                    let running = this.server_flag.load(Ordering::Relaxed);
                    if this.device != latest || this.server_running != running {
                        this.device = latest;
                        this.server_running = running;
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }

                cx.background_executor().timer(REFRESH_INTERVAL).await;
            }
        });

        Self {
            server_running: server_flag.load(Ordering::Relaxed),
            server_flag,
            port_taken,
            device: None,
            logo: Arc::new(Image::from_bytes(ImageFormat::Png, LOGO_PNG.to_vec())),
            _poll: poll,
        }
    }
}

impl Render for StarterView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().border;

        let header = h_flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_2()
            .border_b_1()
            .border_color(border)
            .child(img(self.logo.clone()).size(px(36.)))
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(BRAND_COLOR))
                    .child(format!("Kodular Starter {}", env!("CARGO_PKG_VERSION"))),
            );

        let server_status = if self.port_taken {
            Tag::danger().rounded_full().child(format!(
                "● Port {} is in use by another program",
                crate::server::PORT
            ))
        } else if self.server_running {
            Tag::success()
                .rounded_full()
                .child("● Local server is running")
        } else {
            Tag::danger()
                .rounded_full()
                .child("● Local server is not running")
        };

        let device_section = match &self.device {
            Some(info) => v_flex().gap_2().child("Device is connected via USB").child(
                GroupBox::new().title("Device Info").child(
                    DescriptionList::horizontal()
                        .columns(1)
                        .item("Serial No", info.serial_no.clone(), 1)
                        .item("Model", info.model.clone(), 1)
                        .item("Android Version", info.android_version.clone(), 1)
                        .item("SDK Version", info.sdk_version.clone(), 1)
                        .item(
                            "Companion",
                            info.companion_version
                                .clone()
                                .unwrap_or_else(|| "Not installed".into()),
                            1,
                        ),
                ),
            ),
            None => v_flex().child("Connect your device via USB to see device info"),
        };

        let main = v_flex()
            .flex_1()
            .gap_4()
            .p_4()
            .child(h_flex().child(server_status))
            .child(device_section);

        let footer = h_flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_2()
            .border_t_1()
            .border_color(border)
            .text_sm()
            .child("© Junnovate Limited")
            .child(div().flex_1())
            .child(
                Link::new("guide")
                    .href("https://docs.kodular.io/guides/live-development/usb/")
                    .child("Guide"),
            )
            .child(
                Link::new("source")
                    .href("https://github.com/Kodular/Starter")
                    .child("Source Code"),
            )
            .when_some(crate::logging::log_dir(), |this, dir| {
                this.child(
                    Link::new("logs")
                        .on_click(move |_, _, cx| {
                            let _ = std::fs::create_dir_all(&dir);
                            cx.open_with_system(&dir);
                        })
                        .child("Logs"),
                )
            });

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(header)
            .child(main)
            .child(footer)
    }
}
