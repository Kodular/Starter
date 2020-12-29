module binary

import os

fn extract_adbwinapidll(temp_dir string) {
	mut arr := []byte{len: adbwinapi_dll_len}
	unsafe {C.memcpy(arr.data, adbwinapi_dll, adbwinapi_dll_len)}
	path := os.join_path(temp_dir, 'AdbWinApi.dll')
	mut f := os.open_file(path, 'wb+') or { panic(err) }
	f.write(arr)
	f.close()
}

fn extract_adbwinusbapidll(temp_dir string) {
	mut arr := []byte{len: adbwinusbapi_dll_len}
	unsafe {C.memcpy(arr.data, adbwinusbapi_dll, adbwinusbapi_dll_len)}
	path := os.join_path(temp_dir, 'AdbWinUsbApi.dll')
	mut f := os.open_file(path, 'wb+') or { panic(err) }
	f.write(arr)
	f.close()
}