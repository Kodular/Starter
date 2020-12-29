module binary

import os

fn init_temp_dir() string {
	temp_dir := os.join_path(os.temp_dir(), 'KodularStarter')
	os.rmdir(temp_dir)
	os.mkdir(temp_dir)
	return temp_dir
}

pub fn extract_exe() string {
	temp_dir := init_temp_dir()
	$if windows {
		extract_adbwinapidll(temp_dir)
		extract_adbwinusbapidll(temp_dir)
		mut arr := []byte{len: adb_exe_len}
		unsafe {C.memcpy(arr.data, adb_exe, adb_exe_len)}
		exe_path := os.join_path(temp_dir, 'adb.exe')
		mut f := os.open_file(exe_path, 'wb+') or { panic(err) }
		f.write(arr)
		f.close()
		return exe_path
	} $else {
		mut arr := []byte{len: adb_len}
		unsafe {C.memcpy(arr.data, adb, adb_len)}
		exe_path := os.join_path(temp_dir, 'adb')
		mut f := os.open_file(exe_path, 'wb+') or { panic(err) } // set executable?
		f.write(arr)
		f.close()
		return exe_path
	}
}
