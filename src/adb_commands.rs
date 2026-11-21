use adb_client::{ADBDeviceExt, usb::ADBUSBDevice};
use std::io::stdout;
use std::str::from_utf8;

#[derive(Clone)]
pub(crate) struct DeviceInfo {
    pub(crate) serial_no: String,
    pub(crate) model: String,
    pub(crate) android_version: String,
    pub(crate) sdk_version: String,
    /// `versionName` of the Kodular Companion app, or `None` if it isn't installed.
    pub(crate) companion_version: Option<String>,
}

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";

pub(crate) fn get_connected_device() -> Option<ADBUSBDevice> {
    let autodetect = ADBUSBDevice::autodetect();
    match autodetect {
        Ok(device) => Some(device),
        Err(err) => {
            // Expected whenever no phone is plugged in, and polled every few seconds.
            tracing::debug!("no USB device: {err}");
            None
        }
    }
}

fn shell_output(device: &mut ADBUSBDevice, command: &str) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();
    device.shell_command(&command, Some(&mut buf), None).ok()?;
    from_utf8(&buf).ok().map(|data| data.trim().to_string())
}

fn getprop_from_device(device: &mut ADBUSBDevice, property: &str) -> Option<String> {
    shell_output(device, &format!("getprop {property}"))
}

fn parse_version_name(dumpsys: &str) -> Option<String> {
    dumpsys
        .lines()
        .find_map(|line| line.trim().strip_prefix("versionName="))
        .map(str::trim)
        .filter(|version| !version.is_empty())
        .map(str::to_string)
}

pub(crate) fn get_device_serial(device: &mut ADBUSBDevice) -> Option<String> {
    getprop_from_device(device, "ro.serialno")
}

/// All device info in one shell round trip: four `getprop` lines (each prints
/// exactly one line, empty if unset), then the companion's `versionName=` line if installed.
pub(crate) fn get_device_info(device: &mut ADBUSBDevice) -> Result<DeviceInfo, ()> {
    let command = format!(
        "getprop ro.serialno; getprop ro.product.model; getprop ro.build.version.release; \
         getprop ro.build.version.sdk; dumpsys package {COMPANION_PKG_NAME} | grep -m1 versionName="
    );
    let mut buf: Vec<u8> = Vec::new();
    device.shell_command(&command, Some(&mut buf), None).map_err(|_| ())?;
    parse_device_info(from_utf8(&buf).map_err(|_| ())?).ok_or(())
}

fn parse_device_info(output: &str) -> Option<DeviceInfo> {
    let mut lines = output.lines().map(str::trim);
    let mut prop = || lines.next().filter(|value| !value.is_empty()).map(str::to_string);
    let serial_no = prop()?;
    let model = prop()?;
    let android_version = prop()?;
    let sdk_version = prop()?;
    let rest: Vec<&str> = lines.collect();
    Some(DeviceInfo {
        serial_no,
        model,
        android_version,
        sdk_version,
        companion_version: parse_version_name(&rest.join("\n")),
    })
}

pub(crate) fn start_companion(device_serial: &str) -> Result<(), ()> {
    if let Some(mut device) = get_connected_device()
        && let Some(serial_no) = get_device_serial(&mut device)
    {
        if serial_no != device_serial {
            tracing::warn!(requested = device_serial, connected = %serial_no, "companion start for a different device");
            return Err(());
        }

        tracing::info!(serial = device_serial, "starting companion app");
        if let Err(err) = device.shell_command(
            &format!(
                "am start -a android.intent.action.MAIN -n {}/.Screen1 --ez rundirect true",
                COMPANION_PKG_NAME
            ),
            Some(&mut stdout()),
            None,
        ) {
            tracing::error!(serial = device_serial, "failed to start companion app: {err}");
            return Err(());
        }
    } else {
        tracing::warn!(requested = device_serial, "companion start requested but no device found");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs a USB device"]
    fn bench_device_info() {
        use std::time::{Duration, Instant};
        const RUNS: u32 = 20;

        let mut device = get_connected_device().expect("no device");
        get_device_info(&mut device).unwrap(); // warm-up
        let total: Duration = (0..RUNS)
            .map(|_| {
                let t = Instant::now();
                get_device_info(&mut device).unwrap();
                t.elapsed()
            })
            .sum();
        println!("get_device_info mean: {:?}", total / RUNS);
    }

    #[test]
    fn test_parse_device_info() {
        let info = parse_device_info("a9ebc76e\nV2025\n13\n33\n    versionName=2026.05\n").unwrap();
        assert_eq!(info.serial_no, "a9ebc76e");
        assert_eq!(info.sdk_version, "33");
        assert_eq!(info.companion_version.as_deref(), Some("2026.05"));

        let info = parse_device_info("a9ebc76e\r\nV2025\r\n13\r\n33\r\n").unwrap();
        assert_eq!(info.companion_version, None);

        assert!(parse_device_info("a9ebc76e\n\n13\n33\n").is_none());
    }

    #[test]
    fn test_parse_version_name() {
        assert_eq!(parse_version_name("    versionName=2.71.1\n"), Some("2.71.1".into()));
        assert_eq!(parse_version_name(""), None);
        assert_eq!(parse_version_name("Unable to find package: io.makeroid.companion"), None);
    }

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
