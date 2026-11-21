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

use crate::adb_commands::{DeviceInfo, DeviceTransport, get_connected_device, get_device_info};

const POLL_INTERVAL: Duration = Duration::from_secs(3);
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
    pub fn new(server_flag: Arc<AtomicBool>, port_taken: bool, cx: &mut Context<Self>) -> Self {
        let poll = cx.spawn(async move |this, cx| {
            loop {
                // adb USB calls block, so keep them off the UI thread.
                let device = cx
                    .background_executor()
                    .spawn(async { read_device_info() })
                    .await;

                let alive = this.update(cx, |this, cx| {
                    this.server_running = this.server_flag.load(Ordering::Relaxed);
                    log_device_change(this.device.as_ref(), device.as_ref());
                    this.device = device;
                    cx.notify();
                });
                if alive.is_err() {
                    break;
                }

                cx.background_executor().timer(POLL_INTERVAL).await;
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

fn log_device_change(old: Option<&DeviceInfo>, new: Option<&DeviceInfo>) {
    match (old, new) {
        (None, Some(info)) => tracing::info!(
            serial = %info.serial_no,
            model = %info.model,
            android = %info.android_version,
            sdk = %info.sdk_version,
            "device connected"
        ),
        (Some(info), None) => tracing::info!(serial = %info.serial_no, "device disconnected"),
        (Some(old), Some(new)) if old.serial_no != new.serial_no => tracing::info!(
            from = %old.serial_no,
            to = %new.serial_no,
            "device changed"
        ),
        _ => {}
    }
}

fn read_device_info() -> Option<DeviceInfo> {
    get_connected_device().and_then(|mut device| get_device_info(&mut device).ok())
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
            Tag::success().rounded_full().child("● Local server is running")
        } else {
            Tag::danger().rounded_full().child("● Local server is not running")
        };

        let device_section = match &self.device {
            Some(info) => v_flex()
                .gap_2()
                .child(format!(
                    "Device is connected via {}",
                    match info.transport {
                        DeviceTransport::USB => "USB",
                        DeviceTransport::TCP => "WiFi",
                    }
                ))
                .child(
                    GroupBox::new().title("Device Info").child(
                        DescriptionList::horizontal()
                            .columns(1)
                            .item("Serial No", info.serial_no.clone(), 1)
                            .item("Model", info.model.clone(), 1)
                            .item("Android Version", info.android_version.clone(), 1)
                            .item("SDK Version", info.sdk_version.clone(), 1),
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
