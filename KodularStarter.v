import binary
import os
import vweb

const (
	version            = 2
	companion_pkg_name = 'io.makeroid.companion'
	port               = 8004
	adb_path           = binary.extract_exe()
)

struct App {
pub mut:
	vweb vweb.Context
}

fn main() {
	C.atexit(kill_adb)
	print_info()
	vweb.run<App>(port)
}

pub fn (mut app App) init_once() {
	println('Hit Ctrl+C to quit.')
	println('- '.repeat(22))
	println('[INFO] Turn USB debugging on from the Developer Options on your phone')
}

pub fn (mut app App) init() {
}

fn (mut app App) set_cors_headers() {
	app.vweb.add_header('Access-Control-Allow-Origin', '*')
	app.vweb.add_header('Access-Control-Allow-Headers', 'origin, content-type')
}

pub fn (mut app App) index() vweb.Result {
	app.set_cors_headers()
	return app.vweb.text('Hello VVorld :V')
}

['/ping']
pub fn (mut app App) ping() vweb.Result {
	app.set_cors_headers()
	return app.vweb.json('{"status":"OK","version":$version}')
}

['/utest']
pub fn (mut app App) utest() vweb.Result {
	app.set_cors_headers()
	println('[INFO] Testing...')
	device := get_device() or {
		println('[ERROR] Test failed!')
		return app.vweb.json('{"status":"NO","version":$version}')
	}
	return app.vweb.json('{"status":"OK","version":$version,"device":"$device"}')
}

['/ucheck']
pub fn (mut app App) ucheck() vweb.Result {
	app.set_cors_headers()
	device := get_device() or { return app.vweb.json('{"status":"NO","version":$version}') }
	return app.vweb.json('{"status":"OK","version":$version,"device":"$device"}')
}

['/reset']
pub fn (mut app App) reset() vweb.Result {
	app.set_cors_headers()
	println('[INFO] Resetting...')
	kill_adb()
	return app.vweb.json('{"status":"OK","version":$version}')
}

['/replstart/:deviceid']
pub fn (mut app App) replstart(deviceid string) vweb.Result {
	print('>> Starting companion app on device [$deviceid] (Keep your phone connected via USB)')
	start_companion(deviceid)
	return app.vweb.text('')
}

fn get_device() ?string {
	result := os.exec('$adb_path devices') or {
		println('[ERROR] Failed to retrieve connected devices!')
		return none
	}
	lines := result.output.split_into_lines()
	for line in lines {
		if line.starts_with('*') || line.starts_with('emulator') || 'offline' in line {
			continue
		}
		if line.ends_with('device') {
			device_id := line.all_before('\t')
			println('[INFO] Device found = $device_id')
			return device_id
		}
	}
	println('[ERROR] No devices connected!')
	return none
}

fn start_companion(deviceid string) {
	os.system('$adb_path -s $deviceid forward tcp:8001 tcp:8001')
	os.system('$adb_path -s $deviceid shell am start -a android.intent.action.VIEW -n $companion_pkg_name/.Screen1 --ez rundirect true')
}

fn kill_adb() {
	os.exec('$adb_path kill-server') or {
		println('[ERROR] Failed to kill adb!')
		return
	}
	println('[INFO] Killed adb...')
}

fn print_info() {
	println('Kodular Starter v$version')
	println('OS: $os.user_os()')
	println('ADB path: $adb_path')
	println('- '.repeat(22))
}
