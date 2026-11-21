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

fn get_device_serial(device: &mut ADBUSBDevice) -> Option<String> {
    getprop_from_device(device, "ro.serialno")
}

/// The connected USB device together with its serial number.
pub(crate) fn get_connected_device_with_serial() -> Option<(ADBUSBDevice, String)> {
    let mut device = get_connected_device()?;
    let serial = get_device_serial(&mut device)?;
    Some((device, serial))
}

/// All device info in one shell round trip: four `getprop` lines (each prints
/// exactly one line, empty if unset), then the companion's `versionCode=`/`versionName=` lines if installed.
fn get_device_info(device: &mut ADBUSBDevice) -> Result<DeviceInfo, ()> {
    let command = format!(
        "getprop ro.serialno; getprop ro.product.model; getprop ro.build.version.release; \
         getprop ro.build.version.sdk; dumpsys package {COMPANION_PKG_NAME} | grep -m2 -E 'versionCode=|versionName='"
    );
    let mut buf: Vec<u8> = Vec::new();
    device.shell_command(&command, Some(&mut buf), None).map_err(|_| ())?;
    parse_device_info(from_utf8(&buf).map_err(|_| ())?).ok_or(())
}

/// The parts of the device state that can change while it stays connected:
/// the serial (to notice a different phone) and the companion version.
/// One shell round trip; about half the cost of [`get_device_info`].
fn get_device_status(device: &mut ADBUSBDevice) -> Option<(String, Option<String>)> {
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

/// Polls the connected device, reusing `cached` when it is still the same phone.
///
/// Model, Android and SDK versions can't change while a phone stays connected,
/// so they are fetched only for a newly connected (or different) device. After
/// that, each poll only checks the serial and the companion version.
pub(crate) fn refresh_device_info(cached: Option<DeviceInfo>) -> Option<DeviceInfo> {
    let mut device = get_connected_device()?;
    match cached {
        Some(mut info) => {
            let (serial, companion_version) = get_device_status(&mut device)?;
            if serial == info.serial_no {
                info.companion_version = companion_version;
                Some(info)
            } else {
                get_device_info(&mut device).ok()
            }
        }
        None => get_device_info(&mut device).ok(),
    }
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
    if let Some((device, serial_no)) = get_connected_device_with_serial() {
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
            println!("{name:<20} median {:?}", samples[RUNS as usize / 2]);
        };

        time("get_device_info", &mut |d| {
            get_device_info(d).unwrap();
        });
        time("get_device_status", &mut |d| {
            get_device_status(d).unwrap();
        });

        // Full poll as the UI does it, including USB detect + auth.
        let info = get_device_info(&mut device).unwrap();
        drop(device);
        let poll = |cached: Option<DeviceInfo>| {
            let t = Instant::now();
            refresh_device_info(cached).unwrap();
            t.elapsed()
        };
        let mut first: Vec<Duration> = (0..RUNS).map(|_| poll(None)).collect();
        let mut steady: Vec<Duration> = (0..RUNS).map(|_| poll(Some(info.clone()))).collect();
        first.sort();
        steady.sort();
        println!("{:<20} median {:?}", "poll (connect)", first[RUNS as usize / 2]);
        println!("{:<20} median {:?}", "poll (steady)", steady[RUNS as usize / 2]);
    }

    #[test]
    fn test_parse_device_status() {
        assert_eq!(
            parse_device_status("a9ebc76e\n    versionCode=405 minSdk=21\n    versionName=2026.05\n"),
            Some(("a9ebc76e".into(), Some("2026.05 (405)".into())))
        );
        assert_eq!(parse_device_status("a9ebc76e\r\n"), Some(("a9ebc76e".into(), None)));
        assert_eq!(parse_device_status("a9ebc76e"), Some(("a9ebc76e".into(), None)));
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
        if let Some((_, serial_no)) = get_connected_device_with_serial() {
            start_companion(&serial_no).unwrap();
        }
    }
}
