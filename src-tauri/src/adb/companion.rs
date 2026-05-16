use super::device::AdbDevice;
use super::error::{AdbError, Result};
use super::types::AdbConnectionStrategy;
use std::io::stdout;

const COMPANION_PKG_NAME: &str = "io.makeroid.companion";

/// Launch the companion app on the connected device.
///
/// Performs port forwarding (if using system ADB) and executes the app launch command.
/// Note: USB devices don't support port forwarding yet in adb_client, so the companion
/// app launches but its tcp:8001 socket back to the IDE will not work in BuiltinUsbOnly mode.
pub fn start_companion(strategy: &AdbConnectionStrategy) -> Result<()> {
    let mut device =
        super::device::get_connected_device(strategy).ok_or(AdbError::NoDeviceConnected)?;

    // adb_client does not support port forwarding for USB devices yet
    // (see https://github.com/cocool97/adb_client/issues/63).
    // For BuiltinUsbOnly mode we skip the forward entirely — the companion app
    // will launch but its tcp:8001 socket back to the IDE will not work.
    match &mut device {
        AdbDevice::Server(dev) => {
            if let Err(e) = dev.forward("tcp:8001".to_string(), "tcp:8001".to_string()) {
                log::warn!("Failed to forward port tcp:8001: {:?}", e);
            }
        }
        AdbDevice::Usb(_) => {
            log::warn!(
                "tcp:8001 port forward skipped in BuiltinUsbOnly mode — adb_client lacks USB forward support"
            );
        }
    }

    if let Err(e) = device.shell_command(
        &format!(
            "am start -a android.intent.action.VIEW -n {}/.Screen1 --ez rundirect true",
            COMPANION_PKG_NAME
        ),
        Some(&mut stdout()),
        None,
    ) {
        log::error!("Failed to launch companion app: {:?}", e);
    }

    Ok(())
}
