use adb_client::{ADBDeviceExt, ADBUSBDevice};
use serde::Serialize;
use std::io::stdout;
use std::str::from_utf8;

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

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";

pub(crate) fn get_connected_device() -> Option<ADBUSBDevice> {
    let autodetect = ADBUSBDevice::autodetect();
    match autodetect {
        Ok(device) => Some(device),
        Err(what) => {
            println!("Error: {:?}", what);
            None
        },
    }
}

fn getprop_from_device(device: &mut ADBUSBDevice, property: &str) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();

    match device.shell_command(&["getprop", property], &mut buf) {
        Ok(..) => match from_utf8(buf.as_slice()) {
            Ok(data) => Some(data.trim().to_string()),
            Err(..) => None,
        },
        Err(..) => None,
    }
}

pub(crate) fn get_device_serial(device: &mut ADBUSBDevice) -> Option<String> {
    getprop_from_device(device, "ro.serialno")
}

pub(crate) fn get_device_model(device: &mut ADBUSBDevice) -> Option<String> {
    getprop_from_device(device, "ro.product.model")
}

pub(crate) fn get_device_android_version(device: &mut ADBUSBDevice) -> Option<String> {
    getprop_from_device(device, "ro.build.version.release")
}

pub(crate) fn get_device_sdk_version(device: &mut ADBUSBDevice) -> Option<String> {
    getprop_from_device(device, "ro.build.version.sdk")
}

pub(crate) fn get_device_info(device: &mut ADBUSBDevice) -> Result<DeviceInfo, ()> {
    if let Some(serial_no) = get_device_serial(device) {
        if let Some(model) = get_device_model(device) {
            if let Some(android_version) = get_device_android_version(device) {
                if let Some(sdk_version) = get_device_sdk_version(device) {
                    return Ok(DeviceInfo {
                        transport: DeviceTransport::USB,
                        serial_no,
                        model,
                        android_version,
                        sdk_version,
                    });
                }
            }
        }
    }
    Err(())
}

pub(crate) fn start_companion(device_serial: &str) -> Result<(), ()> {
    if let Some(mut device) = get_connected_device() {
        if let Some(serial_no) = get_device_serial(&mut device) {
            if serial_no != device_serial {
                return Err(());
            }

            let _ = device.shell_command(
                &[
                    "am",
                    "start",
                    "-a",
                    "android.intent.action.MAIN",
                    "-n",
                    &format!("{}/.Screen1", COMPANION_PKG_NAME),
                    "--ez",
                    "rundirect",
                    "true",
                ],
                &mut stdout(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_device_info() {
        if let Some(mut device) = get_connected_device() {
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
        if let Some(mut device) = get_connected_device() {
            if let Some(serial_no) = get_device_serial(&mut device) {
                start_companion(&serial_no).unwrap();
            }
        }
    }
}
