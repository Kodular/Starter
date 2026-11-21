//! Single-instance guard, independent of the server port.
//!
//! An exclusive OS file lock decides which process is the primary instance;
//! the OS releases it if the app exits or crashes. The primary also listens on
//! an ephemeral loopback port (recorded next to the lock) so a later launch can
//! ask it to bring its window to the front before exiting.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;

const FOCUS_REQUEST: &str = "kodular-starter:focus";
const FOCUS_ACK: &str = "kodular-starter:ok";

pub enum Startup {
    /// This process is the only instance. Keep the guard alive for the app's lifetime.
    Primary(InstanceGuard),
    /// Another instance is running; it has been asked to show its window.
    AlreadyRunning,
}

pub struct InstanceGuard {
    _lock: Option<File>,
}

impl InstanceGuard {
    /// Accepts focus requests from later launches and forwards them to `focus_tx`.
    pub fn listen_for_focus(&self, focus_tx: UnboundedSender<()>) {
        let Ok(listener) = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)) else {
            return;
        };
        let Ok(addr) = listener.local_addr() else {
            return;
        };
        let _ = fs::write(port_path(), addr.port().to_string());

        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                if handle_focus_request(stream) && focus_tx.send(()).is_err() {
                    break;
                }
            }
        });
    }
}

pub fn claim() -> Startup {
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(lock_path());

    match lock {
        Ok(file) => match file.try_lock() {
            Ok(()) => Startup::Primary(InstanceGuard { _lock: Some(file) }),
            Err(fs::TryLockError::WouldBlock) => {
                // Even if the running instance can't be reached, don't start a second one.
                if !request_focus() {
                    eprintln!("Kodular Starter is already running.");
                }
                Startup::AlreadyRunning
            }
            Err(fs::TryLockError::Error(err)) => {
                eprintln!("Single-instance lock unavailable: {err}");
                Startup::Primary(InstanceGuard { _lock: None })
            }
        },
        Err(err) => {
            eprintln!("Single-instance lock unavailable: {err}");
            Startup::Primary(InstanceGuard { _lock: None })
        }
    }
}

fn runtime_dir() -> PathBuf {
    std::env::temp_dir()
}

fn lock_path() -> PathBuf {
    runtime_dir().join("io.kodular.starter.lock")
}

fn port_path() -> PathBuf {
    runtime_dir().join("io.kodular.starter.port")
}

fn handle_focus_request(stream: TcpStream) -> bool {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let mut line = String::new();
    if BufReader::new(&stream).read_line(&mut line).is_err() || line.trim() != FOCUS_REQUEST {
        return false;
    }
    let _ = (&stream).write_all(format!("{FOCUS_ACK}\n").as_bytes());
    true
}

fn request_focus() -> bool {
    let Some(port) = fs::read_to_string(port_path())
        .ok()
        .and_then(|s| s.trim().parse::<u16>().ok())
    else {
        return false;
    };

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let timeout = Duration::from_secs(1);
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    if stream
        .write_all(format!("{FOCUS_REQUEST}\n").as_bytes())
        .is_err()
    {
        return false;
    }

    let mut reply = String::new();
    BufReader::new(&stream).read_line(&mut reply).is_ok() && reply.trim() == FOCUS_ACK
}
