use adb_client::{ADBDeviceExt, ADBUSBDevice};
use std::io::stdout;
use std::str::from_utf8;

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";

pub(crate) fn get_connected_device() -> Option<ADBUSBDevice> {
    ADBUSBDevice::autodetect().ok()
}

pub(crate) fn get_device_serial(device: &mut ADBUSBDevice) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();
    device
        .shell_command(["getprop", "ro.serialno"], &mut buf)
        .unwrap();
    let serial_no = from_utf8(buf.as_slice()).unwrap().trim();

    Some(serial_no.to_string())
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

        let mut serial_no = Vec::new();
        let mut model = Vec::new();
        let mut release = Vec::new();
        let mut sdk_version = Vec::new();

        device
            .shell_command(["getprop", "ro.serialno"], &mut serial_no)
            .unwrap();

        device
            .shell_command(["getprop", "ro.product.model"], &mut model)
            .unwrap();

        device
            .shell_command(["xgetprop", "ro.build.version.release"], &mut release)
            .unwrap();

        device
            .shell_command(["getprop", "ro.build.version.sdk"], &mut sdk_version)
            .unwrap();

        println!("Serial No: {:?}", from_utf8(serial_no.as_slice()));
        println!("Model: {:?}", from_utf8(model.as_slice()));
        println!("Release: {:?}", from_utf8(release.as_slice()));
        println!("SDK Version: {:?}", from_utf8(sdk_version.as_slice()));
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
