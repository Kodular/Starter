use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::description_list::DescriptionList;
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants as _};
use gpui_kit::component::link::Link;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::*;

use crate::adb_commands::{DeviceInfo, DeviceTransport, get_connected_device, get_device_info};

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const BRAND_COLOR: u32 = 0x4629a0;
const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

pub struct StarterView {
    server_running: bool,
    device: Option<DeviceInfo>,
    logo: Arc<Image>,
    _poll: Task<()>,
}

impl StarterView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let poll = cx.spawn(async move |this, cx| {
            loop {
                // adb USB calls and the TCP probe block, so keep them off the UI thread.
                let (server_running, device) = cx
                    .background_executor()
                    .spawn(async { (is_server_running(), read_device_info()) })
                    .await;

                let alive = this.update(cx, |this, cx| {
                    this.server_running = server_running;
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
            server_running: false,
            device: None,
            logo: Arc::new(Image::from_bytes(ImageFormat::Png, LOGO_PNG.to_vec())),
            _poll: poll,
        }
    }
}

fn is_server_running() -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], crate::server::PORT));
    TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok()
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

        let server_status = if self.server_running {
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
                    GroupBox::new().outline().title("Device Info").child(
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
            );

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(header)
            .child(main)
            .child(footer)
    }
}
