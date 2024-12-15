use adb_client::{ADBDeviceExt, ADBUSBDevice};
use std::io::stdout;
use std::str::from_utf8;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct DeviceInfo {
    serial_no: String,
    model: String,
    android_version: String,
    sdk_version: String,
}

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";

pub(crate) fn get_connected_device() -> Option<ADBUSBDevice> {
    ADBUSBDevice::autodetect().ok()
}

fn getprop_from_device(device: &mut ADBUSBDevice, property: &str) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();

    device
        .shell_command(["getprop", property], &mut buf)
        .unwrap();

    Some(from_utf8(buf.as_slice()).unwrap().trim().to_string())
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
    let serial_no = get_device_serial(device).unwrap();
    let model = get_device_model(device).unwrap();
    let android_version = get_device_android_version(device).unwrap();
    let sdk_version = get_device_sdk_version(device).unwrap();

    Ok(DeviceInfo {
        serial_no,
        model,
        android_version,
        sdk_version,
    })
}

pub(crate) fn start_companion(device_serial: &str) -> Result<(), ()> {
    let mut device = get_connected_device().unwrap();
    let serial_no = get_device_serial(&mut device).unwrap();

    if serial_no != device_serial {
        return Err(());
    }

    device
        .shell_command(
            [
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
            stdout(),
        )
        .unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_device_info() {
        let mut device = get_connected_device().unwrap();

        let device_info = get_device_info(&mut device);

        println!("Serial No: {:?}", device_info.serial_no);
        println!("Model: {:?}", device_info.model);
        println!("Android Version: {:?}", device_info.android_version);
        println!("SDK Version: {:?}", device_info.sdk_version);
    }

    #[test]
    fn test_start_companion() {
        match get_connected_device() {
            Some(mut device) => {
                let serial_no = get_device_serial(&mut device).unwrap();

                start_companion(&serial_no).unwrap();
            }
            None => {}
        };
    }
}
