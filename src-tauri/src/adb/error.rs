use std::fmt;

/// Errors that can occur during ADB operations.
#[derive(Debug, Clone)]
pub enum AdbError {
    /// No device is connected.
    NoDeviceConnected,
    /// Failed to execute a shell command.
    ShellCommandFailed(String),
    /// Device communication error.
    DeviceError(String),
    /// Failed to parse device output.
    ParseError(String),
}

impl fmt::Display for AdbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdbError::NoDeviceConnected => write!(f, "No device connected"),
            AdbError::ShellCommandFailed(cmd) => write!(f, "Shell command failed: {}", cmd),
            AdbError::DeviceError(msg) => write!(f, "Device error: {}", msg),
            AdbError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for AdbError {}

pub type Result<T> = std::result::Result<T, AdbError>;
