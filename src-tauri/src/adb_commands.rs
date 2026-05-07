use crate::settings::AppSettings;
use adb_client::{
    ADBDeviceExt, server::ADBServer, server_device::ADBServerDevice, usb::ADBUSBDevice,
};
use serde::{Deserialize, Serialize};
use std::io::{Write, stdout};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::from_utf8;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AdbMode {
    #[default]
    Auto,
    System,
    Builtin,
}

#[derive(Serialize)]
pub(crate) enum DeviceTransport {
    USB,
    TCP,
}

#[derive(Serialize)]
pub(crate) struct DeviceInfo {
    transport: DeviceTransport,
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
    fn shell_command_inner(
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

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";

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
    let adb_path = settings.custom_adb_path.clone();
    match settings.adb_mode {
        AdbMode::Auto => {
            if let Some(device) = try_system_adb(adb_path) {
                return Some(device);
            }
            match ADBUSBDevice::autodetect() {
                Ok(device) => Some(AdbDevice::Usb(device)),
                Err(what) => {
                    println!("Error: {:?}", what);
                    None
                }
            }
        }
        AdbMode::System => try_system_adb(adb_path),
        AdbMode::Builtin => match ADBUSBDevice::autodetect() {
            Ok(device) => Some(AdbDevice::Usb(device)),
            Err(what) => {
                println!("Error: {:?}", what);
                None
            }
        },
    }
}

fn getprop_from_device(device: &mut AdbDevice, property: &str) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();

    match device.shell_command_inner(&format!("getprop {}", property), Some(&mut buf), None) {
        Ok(..) => match from_utf8(buf.as_slice()) {
            Ok(data) => Some(data.trim().to_string()),
            Err(..) => None,
        },
        Err(..) => None,
    }
}

pub(crate) fn get_device_serial(device: &mut AdbDevice) -> Option<String> {
    getprop_from_device(device, "ro.serialno")
}

pub(crate) fn get_device_model(device: &mut AdbDevice) -> Option<String> {
    getprop_from_device(device, "ro.product.model")
}

pub(crate) fn get_device_android_version(device: &mut AdbDevice) -> Option<String> {
    getprop_from_device(device, "ro.build.version.release")
}

pub(crate) fn get_device_sdk_version(device: &mut AdbDevice) -> Option<String> {
    getprop_from_device(device, "ro.build.version.sdk")
}

pub(crate) fn get_device_info(device: &mut AdbDevice) -> Result<DeviceInfo, ()> {
    let transport = match device {
        AdbDevice::Usb(_) => DeviceTransport::USB,
        AdbDevice::Server(_) => DeviceTransport::TCP,
    };

    if let Some(serial_no) = get_device_serial(device)
        && let Some(model) = get_device_model(device)
        && let Some(android_version) = get_device_android_version(device)
        && let Some(sdk_version) = get_device_sdk_version(device)
    {
        return Ok(DeviceInfo {
            transport,
            serial_no,
            model,
            android_version,
            sdk_version,
        });
    }
    Err(())
}

pub(crate) fn start_companion(device_serial: &str, settings: &AppSettings) -> Result<(), ()> {
    if let Some(mut device) = get_connected_device(settings)
        && let Some(serial_no) = get_device_serial(&mut device)
    {
        if serial_no != device_serial {
            return Err(());
        }

        let _ = device.shell_command_inner(
            &format!(
                "am start -a android.intent.action.MAIN -n {}/.Screen1 --ez rundirect true",
                COMPANION_PKG_NAME
            ),
            Some(&mut stdout()),
            None,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin_settings() -> crate::settings::AppSettings {
        crate::settings::AppSettings {
            adb_mode: AdbMode::Builtin,
            ..Default::default()
        }
    }

    #[test]
    fn test_get_device_info() {
        let settings = builtin_settings();
        if let Some(mut device) = get_connected_device(&settings) {
            if let Ok(device_info) = get_device_info(&mut device) {
                println!("Serial No: {:?}", device_info.serial_no);
                println!("Model: {:?}", device_info.model);
                println!("Android Version: {:?}", device_info.android_version);
                println!("SDK Version: {:?}", device_info.sdk_version);
            }
        }
    }

    #[test]
    fn test_start_companion() {
        let settings = builtin_settings();
        if let Some(mut device) = get_connected_device(&settings) {
            if let Some(serial_no) = get_device_serial(&mut device) {
                start_companion(&serial_no, &settings).unwrap();
            }
        }
    }
}
