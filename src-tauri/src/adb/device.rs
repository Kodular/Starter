use super::types::{AdbConnectionStrategy, CompanionStatus, DeviceInfo};
use adb_client::{
    ADBDeviceExt, server::ADBServer, server_device::ADBServerDevice, usb::ADBUSBDevice,
};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";
const ADB_SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

pub enum AdbDevice {
    Usb(ADBUSBDevice),
    Server(ADBServerDevice),
}

impl AdbDevice {
    pub fn shell_command(
        &mut self,
        command: &dyn AsRef<str>,
        stdout: Option<&mut dyn Write>,
        stderr: Option<&mut dyn Write>,
    ) -> adb_client::Result<Option<u8>> {
        match self {
            AdbDevice::Usb(dev) => dev.shell_command(command, stdout, stderr),
            AdbDevice::Server(dev) => dev.shell_command(command, stdout, stderr),
        }
    }
}

fn try_system_adb(adb_path: Option<String>) -> Option<AdbDevice> {
    let mut server = ADBServer::new_from_path(ADB_SERVER_ADDR, adb_path);
    if server.version().is_err() {
        return None;
    }
    let device = ADBServerDevice::autodetect(Some(ADB_SERVER_ADDR));
    Some(AdbDevice::Server(device))
}

/// Get the connected ADB device based on the resolved connection strategy.
pub fn get_connected_device(strategy: &AdbConnectionStrategy) -> Option<AdbDevice> {
    match strategy {
        AdbConnectionStrategy::PreferSystemAdb { adb_path } => {
            let path = adb_path.as_ref().map(|p| p.to_string_lossy().into_owned());
            try_system_adb(path).or_else(|| ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb))
        }
        AdbConnectionStrategy::BuiltinUsbOnly => {
            ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb)
        }
    }
}

/// Parse companion app version information from dumpsys output.
fn parse_companion_status(lines: &mut std::str::Lines<'_>) -> CompanionStatus {
    let version_name = lines
        .find(|l| l.contains("versionName="))
        .and_then(|l| l.trim().split_once('='))
        .map(|(_, v)| v.trim().to_string());
    let version_code = lines
        .find(|l| l.contains("versionCode="))
        .and_then(|l| l.trim().split_once('='))
        .and_then(|(_, v)| v.split_whitespace().next().map(str::to_string));
    log::info!(
        "parse_companion_status: version_name={version_name:?} version_code={version_code:?}"
    );
    match (version_name, version_code) {
        (Some(version_name), Some(version_code)) if !version_name.is_empty() => {
            CompanionStatus::Installed {
                version_name,
                version_code,
            }
        }
        _ => {
            log::info!("parse_companion_status: companion app not found or version fields missing");
            CompanionStatus::NotInstalled
        }
    }
}

/// Query device information including model, Android version, and companion app status.
pub fn get_device_info(device: &mut AdbDevice) -> Option<DeviceInfo> {
    let mut buf = Vec::new();
    device
        .shell_command(
            &format!(
                "getprop ro.serialno; \
                 getprop ro.product.model; \
                 getprop ro.build.version.release; \
                 getprop ro.build.version.sdk; \
                 dumpsys package {COMPANION_PKG_NAME} | grep -m1 versionName; \
                 dumpsys package {COMPANION_PKG_NAME} | grep -m1 versionCode"
            ),
            Some(&mut buf),
            None,
        )
        .ok()?;
    let text = std::str::from_utf8(&buf).ok()?;
    let mut lines = text.lines();
    let serial_no = lines.next()?.trim().to_string();
    let model = lines.next()?.trim().to_string();
    let android_version = lines.next()?.trim().to_string();
    let sdk_version = lines.next()?.trim().to_string();
    let companion_status = parse_companion_status(&mut lines);
    Some(DeviceInfo {
        serial_no,
        model,
        android_version,
        sdk_version,
        companion_status,
    })
}
