#!/usr/bin/env python
import platform
import re
import subprocess

from bottle import run, route, response


def hr():
    """ Horizontal Rule """
    print('- ' * 25)


VERSION = '1.0.1-Andromeda'
PACKAGE_NAME = 'io.makeroid.companion'

OS = platform.system()

print(f'Makeroid Starter v{VERSION} for {OS}')
hr()


@route('/ping/')
def ping():
    print('Ping...')
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    return {
        "status": "OK",
        "version": VERSION
    }


@route('/utest/')
def utest():
    print('Testing...')
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    device = checkrunning()
    if device:
        print('Test Successful!')
        return {
            "status": "OK",
            "device": device,
            "version": VERSION
        }
    else:
        print('Test Failed!')
        return {
            "status": "NO",
            "version": VERSION
        }


@route('/ucheck/')
def ucheck():
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    device = checkrunning()
    if device:
        return {
            "status": "OK",
            "device": device,
            "version": VERSION
        }
    else:
        return {
            "status": "NO",
            "version": VERSION
        }


@route('/reset/')
def reset():
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    print('Resetting...')
    shutdown()
    print('Reset Done!')
    return {
        "status": "OK",
        "version": VERSION
    }


@route('/replstart/:device')
def replstart(device=None):
    print(f'Device = {device}')
    print('Starting companion app (Keep your phone connected through USB)')
    try:
        subprocess.check_output(f'adb -s {device} forward tcp:8001 tcp:8001', shell=True)
        subprocess.check_output(
            f'adb -s {device} shell am start -a android.intent.action.VIEW -n {PACKAGE_NAME}/.Screen1 --ez rundirect true',
            shell=True)
        response.headers['Access-Control-Allow-Origin'] = '*'
        response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
        return ''
    except subprocess.CalledProcessError as e:
        print(f'Problem starting companion app : status {e.returncode}\n')
        return ''


def checkrunning():
    global match
    match = ''
    print('Checking device...')
    try:
        result = subprocess.check_output('adb devices', shell=True)
        lines = result.splitlines()
        for line in lines[1:]:
            if line:
                line_str = str(line, 'utf-8')  # convert byte to string
                if re.search(r'(emulator-\d+)\s+device', line_str):  # We are an emulator
                    continue  # Skip it
                match = re.search(r'([\w\d]+)\s+device', line_str)
                if match:
                    break
        if match:
            return match.group(1)
        return False
    except subprocess.CalledProcessError as e:
        print(f'Problem checking for devices : status {e.returncode}')
        return False


def killadb():
    try:
        subprocess.check_output('adb kill-server', shell=True)
        print('Killed adb')
    except subprocess.CalledProcessError as e:
        print(f'Problem stopping adb : status {e.returncode}')
        return ''


def shutdown():
    killadb()


if __name__ == '__main__':
    import atexit
    atexit.register(shutdown)

    run(host='127.0.0.1', port=8004)
