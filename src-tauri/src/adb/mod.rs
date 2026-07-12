pub mod companion;
pub mod device;
pub mod error;
pub mod monitor;
pub mod strategy;
pub mod types;

pub use companion::start_companion;
pub use device::{get_connected_device, get_device_info};
pub use monitor::run as run_monitor;
pub use strategy::{detect_adb_path, kill_adb_server, resolve_adb_strategy, test_adb_path};
pub use types::{AdbConnectionStrategy, AdbState};
