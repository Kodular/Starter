use adb_client::{ADBDeviceExt, usb::ADBUSBDevice};
use std::io::stdout;
use std::str::from_utf8;

#[derive(Clone)]
pub(crate) struct DeviceInfo {
    pub(crate) serial_no: String,
    pub(crate) model: String,
    pub(crate) android_version: String,
    pub(crate) sdk_version: String,
    /// Kodular Companion version as "versionName (versionCode)", or `None` if it isn't installed.
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

/// Formats `dumpsys package` output as "versionName (versionCode)".
///
/// `versionName=` is on a line of its own; `versionCode=` shares a line with
/// other fields (`versionCode=42 minSdk=21 targetSdk=34`).
fn parse_companion_version(dumpsys: &str) -> Option<String> {
    let field = |key: &str| {
        dumpsys.lines().find_map(|line| {
            line.split_whitespace()
                .find_map(|token| token.strip_prefix(key))
                .filter(|value| !value.is_empty())
        })
    };
    let name = dumpsys
        .lines()
        .find_map(|line| line.trim().strip_prefix("versionName="))
        .map(str::trim)
        .filter(|name| !name.is_empty())?;
    Some(match field("versionCode=") {
        Some(code) => format!("{name} ({code})"),
        None => name.to_string(),
    })
}

pub(crate) fn get_device_serial(device: &mut ADBUSBDevice) -> Option<String> {
    getprop_from_device(device, "ro.serialno")
}

/// All device info in one shell round trip: four `getprop` lines (each prints
/// exactly one line, empty if unset), then the companion's `versionCode=`/`versionName=` lines if installed.
pub(crate) fn get_device_info(device: &mut ADBUSBDevice) -> Result<DeviceInfo, ()> {
    let command = format!(
        "getprop ro.serialno; getprop ro.product.model; getprop ro.build.version.release; \
         getprop ro.build.version.sdk; dumpsys package {COMPANION_PKG_NAME} | grep -m2 -E 'versionCode=|versionName='"
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
        companion_version: parse_companion_version(&rest.join("\n")),
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
        let info = parse_device_info(
            "a9ebc76e\nV2025\n13\n33\n    versionCode=202605 minSdk=21 targetSdk=34\n    versionName=2026.05\n",
        )
        .unwrap();
        assert_eq!(info.serial_no, "a9ebc76e");
        assert_eq!(info.sdk_version, "33");
        assert_eq!(info.companion_version.as_deref(), Some("2026.05 (202605)"));

        let info = parse_device_info("a9ebc76e\r\nV2025\r\n13\r\n33\r\n").unwrap();
        assert_eq!(info.companion_version, None);

        assert!(parse_device_info("a9ebc76e\n\n13\n33\n").is_none());
    }

    #[test]
    fn test_parse_companion_version() {
        assert_eq!(
            parse_companion_version("    versionCode=271 minSdk=21 targetSdk=34\n    versionName=2.71.1"),
            Some("2.71.1 (271)".into())
        );
        assert_eq!(parse_companion_version("    versionName=2.71.1\n"), Some("2.71.1".into()));
        assert_eq!(parse_companion_version(""), None);
        assert_eq!(parse_companion_version("Unable to find package: io.makeroid.companion"), None);
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
