# Kodular Starter
![Total Downloads](https://img.shields.io/github/downloads/Kodular/Starter/total.svg) ![GitHub release](https://img.shields.io/github/release/Kodular/Starter.svg) ![CI](https://github.com/Kodular/Starter/workflows/publish/badge.svg)

Kodular Starter to test apps live via USB

![image](https://github.com/user-attachments/assets/4abb2ae7-5d7e-4134-98d3-2fbdea49de9d)

### Installing

Refer the documentation on this page for instructions on downloading and installing:
https://docs.kodular.io/guides/live-development/usb/

### Testing

To test the connection, go to https://starter.kodular.io

### Development

Kodular Starter is a native Rust app built with [GPUI Kit](https://gpui-kit.com). It runs a local server on port 8004 that Kodular Creator talks to, and shows the server and USB device status.

```bash
cargo run
```

On Linux, install the GPUI and libusb build dependencies first (see `.github/workflows/publish.yml`).

### Logs

Logs rotate daily (last 7 days kept), and the **Logs** link in the app footer opens the folder:

- macOS: `~/Library/Logs/Kodular Starter/`
- Windows: `%LOCALAPPDATA%\Kodular Starter\logs\`
- Linux: `~/.local/state/kodular-starter/logs/`

Set `RUST_LOG` for more detail, e.g. `RUST_LOG=debug` or `RUST_LOG=info,tower_http=debug` to log every HTTP request.
