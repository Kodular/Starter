//! The single owner of the USB device.
//!
//! Only this thread talks to the phone. It keeps one connection open across
//! polls (saving USB detect + auth on every poll, and keeping the Companion's
//! port forward alive), publishes a [`DeviceInfo`] snapshot, and runs commands
//! sent by the server. The UI and `/ucheck` read the snapshot and never open
//! the USB interface themselves, so they can't contend for it.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use adb_client::usb::ADBUSBDevice;
use tokio::sync::{oneshot, watch};

use crate::adb_commands::{
    DeviceInfo, get_connected_device, get_device_info, get_device_status, launch_companion,
};

const POLL_INTERVAL: Duration = Duration::from_secs(3);

enum Command {
    StartCompanion {
        serial: String,
        reply: oneshot::Sender<Result<(), ()>>,
    },
}

#[derive(Clone)]
pub struct DeviceHandle {
    state: watch::Receiver<Option<DeviceInfo>>,
    commands: mpsc::Sender<Command>,
}

impl DeviceHandle {
    /// The most recently polled device, if one is connected.
    pub fn current(&self) -> Option<DeviceInfo> {
        self.state.borrow().clone()
    }

    /// Forwards the Companion port and launches the Companion on the device
    /// with `serial`. Fails if that device isn't the one connected.
    pub async fn start_companion(&self, serial: String) -> Result<(), ()> {
        let (reply, response) = oneshot::channel();
        self.commands
            .send(Command::StartCompanion { serial, reply })
            .map_err(|_| ())?;
        response.await.map_err(|_| ())?
    }
}

/// Starts the device thread. It stops when every [`DeviceHandle`] is dropped.
pub fn spawn() -> DeviceHandle {
    let (state_tx, state) = watch::channel(None);
    let (commands, command_rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("device".into())
        .spawn(move || run(state_tx, command_rx))
        .expect("failed to spawn device thread");
    DeviceHandle { state, commands }
}

struct Connection {
    device: ADBUSBDevice,
    info: DeviceInfo,
}

fn run(state_tx: watch::Sender<Option<DeviceInfo>>, commands: mpsc::Receiver<Command>) {
    let mut conn: Option<Connection> = None;
    let mut next_poll = Instant::now();

    loop {
        // Sleep until the next poll, but wake immediately for commands.
        match commands.recv_timeout(next_poll.saturating_duration_since(Instant::now())) {
            Ok(Command::StartCompanion { serial, reply }) => {
                if conn.is_none() {
                    // Plugged in since the last poll: pick it up now.
                    conn = poll(None);
                    publish(&state_tx, &conn);
                }
                let result = match &conn {
                    Some(c) if c.info.serial_no == serial => {
                        tracing::info!(serial, "starting companion app");
                        launch_companion(&c.device)
                    }
                    Some(c) => {
                        tracing::warn!(requested = serial, connected = %c.info.serial_no, "companion start for a different device");
                        Err(())
                    }
                    None => {
                        tracing::warn!(
                            requested = serial,
                            "companion start requested but no device found"
                        );
                        Err(())
                    }
                };
                let _ = reply.send(result);
                continue;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        conn = poll(conn.take());
        publish(&state_tx, &conn);
        next_poll = Instant::now() + POLL_INTERVAL;
    }
}

/// Re-checks an open connection, or connects if there is none.
///
/// Model, Android and SDK versions can't change while a phone stays connected,
/// so an open connection only re-reads the serial (to notice a different phone)
/// and the companion version.
fn poll(conn: Option<Connection>) -> Option<Connection> {
    match conn {
        Some(mut c) => match get_device_status(&mut c.device) {
            Some((serial, companion_version)) if serial == c.info.serial_no => {
                c.info.companion_version = companion_version;
                Some(c)
            }
            // Unplugged, or a different phone: drop the handle (and its forward) and start over.
            _ => connect(),
        },
        None => connect(),
    }
}

fn connect() -> Option<Connection> {
    let mut device = get_connected_device()?;
    let info = get_device_info(&mut device).ok()?;
    Some(Connection { device, info })
}

fn publish(state_tx: &watch::Sender<Option<DeviceInfo>>, conn: &Option<Connection>) {
    let next = conn.as_ref().map(|c| c.info.clone());
    state_tx.send_if_modified(|current| {
        if *current == next {
            return false;
        }
        log_device_change(current.as_ref(), next.as_ref());
        *current = next;
        true
    });
}

fn log_device_change(old: Option<&DeviceInfo>, new: Option<&DeviceInfo>) {
    match (old, new) {
        (None, Some(info)) => tracing::info!(
            serial = %info.serial_no,
            model = %info.model,
            android = %info.android_version,
            sdk = %info.sdk_version,
            companion = info.companion_version.as_deref().unwrap_or("not installed"),
            "device connected"
        ),
        (Some(info), None) => tracing::info!(serial = %info.serial_no, "device disconnected"),
        (Some(old), Some(new)) if old.serial_no != new.serial_no => tracing::info!(
            from = %old.serial_no,
            to = %new.serial_no,
            "device changed"
        ),
        (Some(old), Some(new)) if old.companion_version != new.companion_version => {
            tracing::info!(
                serial = %new.serial_no,
                from = old.companion_version.as_deref().unwrap_or("not installed"),
                to = new.companion_version.as_deref().unwrap_or("not installed"),
                "companion changed"
            )
        }
        _ => {}
    }
}
