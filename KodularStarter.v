import os
import regex
import runtime
import vweb

const (
	version            = 2
	companion_pkg_name = 'io.makeroid.companion'
	port               = 8004
	regex_emulator     = r'(emulator-\d+)\s+device'
	regex_device       = r'(\w+)\s+device'
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
	println('Testing...')
	device := get_device() or {
		println('Test failed!')
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
	println('Resetting...')
	kill_adb()
	return app.vweb.json('{"status":"OK","version":$version}')
}

['/replstart/:deviceid']
pub fn (mut app App) replstart(deviceid string) vweb.Result {
	print('Starting companion app on device [$deviceid] (Keep your phone connected via USB)')
	run_companion(deviceid)
	return app.vweb.text('')
}

fn get_device() ?string {
	result := os.exec('adb devices') or { panic(err) }
	lines := result.output.split_into_lines()
	mut re_emu := regex.regex_opt(regex_emulator) or { panic(err) }
	mut re_dev := regex.regex_opt(regex_device) or { panic(err) }
	for line in lines[1..] {
		if line == '' {
			continue
		}
		if line.starts_with('*') {
			continue
		}
		if 'offline' in line {
			continue
		}
		emu_start, _ := re_emu.match_string(line)
		if emu_start >= 0 {
			continue
		}
		start, _ := re_dev.match_string(line)
		if start >= 0 {
			group := re_dev.get_group_list()[0]
			return line[group.start..group.end]
		}
	}
	return none
}

fn run_companion(deviceid string) {
	os.system('adb -s $deviceid forward tcp:8001 tcp:8001')
	os.system('adb -s $deviceid shell am start -a android.intent.action.VIEW -n $companion_pkg_name/.Screen1 --ez rundirect true')
}

fn kill_adb() {
	os.exec('adb kill-server') or {
		eprintln('Failed to kill adb!')
		return
	}
	println('Killed adb...')
}

fn print_info() {
	os_info := os.uname()
	os_name := os.user_os()
	arch := if runtime.is_64bit() { '64-bit' } else { '32-bit' }
	println('Kodular Starter version: $version')
	println('OS: $os_name')
	println('Architecture: $arch')
	println('Machine: $os_info.machine')
	println('ADB path: null')
	println('- '.repeat(22))
}
