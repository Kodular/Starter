use crate::adb_resolver::resolve_external_adb_path;
use crate::settings::AppSettings;
use adb_client::{
    ADBDeviceExt, server::ADBServer, server_device::ADBServerDevice, usb::ADBUSBDevice,
};
use serde::{Deserialize, Serialize};
use std::io::{Write, stdout};
use std::net::{Ipv4Addr, SocketAddrV4};

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AdbMode {
    #[default]
    Auto,
    Builtin,
}

#[derive(Serialize)]
pub(crate) struct DeviceInfo {
    serial_no: String,
    model: String,
    android_version: String,
    sdk_version: String,
}

pub(crate) enum AdbDevice {
    Usb(ADBUSBDevice),
    Server(ADBServerDevice),
}

impl AdbDevice {
    fn shell_command(
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

pub(crate) fn localhost_addr() -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037)
}

fn try_system_adb(adb_path: Option<String>) -> Option<AdbDevice> {
    let mut server = ADBServer::new_from_path(localhost_addr(), adb_path);
    if server.version().is_err() {
        return None;
    }
    let device = ADBServerDevice::autodetect(Some(localhost_addr()));
    Some(AdbDevice::Server(device))
}

pub(crate) fn get_connected_device(settings: &AppSettings) -> Option<AdbDevice> {
    match settings.adb_mode {
        AdbMode::Auto => {
            let resolved = resolve_external_adb_path(settings.custom_adb_path.as_deref());
            try_system_adb(resolved).or_else(|| ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb))
        }
        AdbMode::Builtin => ADBUSBDevice::autodetect().ok().map(AdbDevice::Usb),
    }
}

fn getprop_from_device(device: &mut AdbDevice, property: &str) -> Option<String> {
    let mut buf = Vec::new();
    device
        .shell_command(&format!("getprop {property}"), Some(&mut buf), None)
        .ok()?;
    std::str::from_utf8(&buf).ok().map(|s| s.trim().to_string())
}

pub(crate) fn get_device_serial(device: &mut AdbDevice) -> Option<String> {
    getprop_from_device(device, "ro.serialno")
}

fn get_device_model(device: &mut AdbDevice) -> Option<String> {
    getprop_from_device(device, "ro.product.model")
}

fn get_device_android_version(device: &mut AdbDevice) -> Option<String> {
    getprop_from_device(device, "ro.build.version.release")
}

fn get_device_sdk_version(device: &mut AdbDevice) -> Option<String> {
    getprop_from_device(device, "ro.build.version.sdk")
}

pub(crate) fn get_device_info(device: &mut AdbDevice) -> Option<DeviceInfo> {
    Some(DeviceInfo {
        serial_no: get_device_serial(device)?,
        model: get_device_model(device)?,
        android_version: get_device_android_version(device)?,
        sdk_version: get_device_sdk_version(device)?,
    })
}

pub(crate) fn start_companion(device_serial: &str, settings: &AppSettings) -> Result<(), ()> {
    let mut device = get_connected_device(settings).ok_or(())?;
    let serial_no = get_device_serial(&mut device).ok_or(())?;
    if serial_no != device_serial {
        return Err(());
    }
    let _ = device.shell_command(
        &format!(
            "am start -a android.intent.action.MAIN -n {}/.Screen1 --ez rundirect true",
            COMPANION_PKG_NAME
        ),
        Some(&mut stdout()),
        None,
    );
    Ok(())
}
