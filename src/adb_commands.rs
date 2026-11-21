use adb_client::{ADBDeviceExt, usb::ADBUSBDevice};
use std::str::from_utf8;

#[derive(Clone, PartialEq)]
pub(crate) struct DeviceInfo {
    pub(crate) serial_no: String,
    pub(crate) model: String,
    pub(crate) android_version: String,
    pub(crate) sdk_version: String,
    /// Kodular Companion version as "versionName (versionCode)", or `None` if it isn't installed.
    pub(crate) companion_version: Option<String>,
}

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";
/// Kodular Creator talks to the Companion's REPL on this port; over USB it is
/// reached through a host -> device forward of the same port.
const COMPANION_REPL_PORT: u16 = 8001;

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

/// All device info in one shell round trip: four `getprop` lines (each prints
/// exactly one line, empty if unset), then the companion's `versionCode=`/`versionName=` lines if installed.
pub(crate) fn get_device_info(device: &mut ADBUSBDevice) -> Result<DeviceInfo, ()> {
    let command = format!(
        "getprop ro.serialno; getprop ro.product.model; getprop ro.build.version.release; \
         getprop ro.build.version.sdk; dumpsys package {COMPANION_PKG_NAME} | grep -m2 -E 'versionCode=|versionName='"
    );
    let mut buf: Vec<u8> = Vec::new();
    device
        .shell_command(&command, Some(&mut buf), None)
        .map_err(|_| ())?;
    parse_device_info(from_utf8(&buf).map_err(|_| ())?).ok_or(())
}

/// The parts of the device state that can change while it stays connected:
/// the serial (to notice a different phone) and the companion version.
/// One shell round trip; about half the cost of [`get_device_info`].
pub(crate) fn get_device_status(device: &mut ADBUSBDevice) -> Option<(String, Option<String>)> {
    let command = format!(
        "getprop ro.serialno; dumpsys package {COMPANION_PKG_NAME} | grep -m2 -E 'versionCode=|versionName='"
    );
    let mut buf: Vec<u8> = Vec::new();
    device.shell_command(&command, Some(&mut buf), None).ok()?;
    parse_device_status(from_utf8(&buf).ok()?)
}

fn parse_device_status(output: &str) -> Option<(String, Option<String>)> {
    let (serial, rest) = output.split_once('\n').unwrap_or((output, ""));
    let serial = serial.trim();
    (!serial.is_empty()).then(|| (serial.to_string(), parse_companion_version(rest)))
}

fn parse_device_info(output: &str) -> Option<DeviceInfo> {
    let mut lines = output.lines().map(str::trim);
    let mut prop = || {
        lines
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
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

/// Forwards the Companion REPL port and launches the Companion in REPL mode.
///
/// On a direct USB connection the forward is relayed by `device` itself, so it
/// only lasts while that handle stays alive (see [`crate::device`]).
pub(crate) fn launch_companion(device: &ADBUSBDevice) -> Result<(), ()> {
    let port = format!("tcp:{COMPANION_REPL_PORT}");
    match device.forward(&port, &port) {
        Ok(local) => tracing::info!(local, "forwarding companion port"),
        // Without the forward Creator can't reach the Companion, but launching it still helps.
        Err(err) => tracing::error!("failed to forward {port}: {err}"),
    }

    // Kodular Companion's Screen1 only declares a MAIN intent filter; Android 13+
    // rejects other actions even for explicit intents.
    let command = format!(
        "am start -a android.intent.action.MAIN -n {COMPANION_PKG_NAME}/.Screen1 --ez rundirect true"
    );
    let mut output = Vec::new();
    if let Err(err) = device.shell_command(&command, Some(&mut output), None) {
        tracing::error!("failed to start companion app: {err}");
        return Err(());
    }
    let output = String::from_utf8_lossy(&output);
    if output.contains("Error") {
        tracing::error!(output = %output.trim(), "failed to start companion app");
        return Err(());
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
        const RUNS: usize = 20;

        let mut device = get_connected_device().expect("no device");
        let mut time = |name: &str, f: &mut dyn FnMut(&mut ADBUSBDevice)| {
            f(&mut device); // warm-up
            let mut samples: Vec<Duration> = (0..RUNS)
                .map(|_| {
                    let t = Instant::now();
                    f(&mut device);
                    t.elapsed()
                })
                .collect();
            samples.sort();
            println!("{name:<20} median {:?}", samples[RUNS / 2]);
        };

        time("get_device_info", &mut |d| {
            get_device_info(d).unwrap();
        });
        time("get_device_status", &mut |d| {
            get_device_status(d).unwrap();
        });
    }

    #[test]
    fn test_parse_device_status() {
        assert_eq!(
            parse_device_status(
                "a9ebc76e\n    versionCode=405 minSdk=21\n    versionName=2026.05\n"
            ),
            Some(("a9ebc76e".into(), Some("2026.05 (405)".into())))
        );
        assert_eq!(
            parse_device_status("a9ebc76e\r\n"),
            Some(("a9ebc76e".into(), None))
        );
        assert_eq!(
            parse_device_status("a9ebc76e"),
            Some(("a9ebc76e".into(), None))
        );
        assert_eq!(parse_device_status("\n"), None);
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
            parse_companion_version(
                "    versionCode=271 minSdk=21 targetSdk=34\n    versionName=2.71.1"
            ),
            Some("2.71.1 (271)".into())
        );
        assert_eq!(
            parse_companion_version("    versionName=2.71.1\n"),
            Some("2.71.1".into())
        );
        assert_eq!(parse_companion_version(""), None);
        assert_eq!(
            parse_companion_version("Unable to find package: io.makeroid.companion"),
            None
        );
    }

    #[test]
    #[ignore = "needs a USB device"]
    fn test_get_device_info() {
        let mut device = get_connected_device().expect("no device");
        let info = get_device_info(&mut device).unwrap();
        println!("Serial No: {:?}", info.serial_no);
        println!("Model: {:?}", info.model);
        println!("Android Version: {:?}", info.android_version);
        println!("SDK Version: {:?}", info.sdk_version);
        println!("Companion: {:?}", info.companion_version);
    }

    #[test]
    #[ignore = "needs a USB device; launches the Companion"]
    fn test_launch_companion() {
        let device = get_connected_device().expect("no device");
        launch_companion(&device).unwrap();
    }
}
