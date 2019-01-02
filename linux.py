#!/usr/bin/env python
import platform
import re
from subprocess import check_call, check_output, call, CalledProcessError
import atexit

from bottle import run, route, response

VERSION = '1.3 Draco'
PACKAGE_NAME = 'io.makeroid.companion'


print('Kodular Starter', 'Version ' + VERSION, ', for Linux')
print('- ' * 31)

# Check for ADB
if call(['which', 'adb']) != 0:
    raise Exception('ADB not installed!')



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
    killadb()
    print('Reset Done!')
    return {
        "status": "OK",
        "version": VERSION
    }


@route('/replstart/<device>')
def replstart(device):
    print('Device =', device)
    print('Starting companion app (Keep your phone connected via USB)')
    try:
        check_call(
            ['adb',
             '-s', device,
             'forward', 'tcp:8001', 'tcp:8001'])
        check_call(
            ['adb',
             '-s', device,
             'shell',
             'am', 'start',
             '-a', 'android.intent.action.VIEW',
             '-n', PACKAGE_NAME + '/.Screen1',
             '--ez', 'rundirect', 'true'])
    except CalledProcessError as e:
        print('Problem starting companion app : status', e.returncode, '\n')

    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    return ''


def checkrunning():
    match = ''
    print('Checking device...')
    try:
        result = check_output(['adb', 'devices'])
        lines = result.decode('utf-8').strip().splitlines()
        for line in lines[1:]:
            if not line or line.startswith('*') or 'offline' in line:
                continue
            if re.search(r'(emulator-\d+)\s+device', line):  # We are an emulator
                continue  # Skip it
            match = re.search(r'(\w+)\s+device', line)
            if match:
                break
    except CalledProcessError as e:
        print('Problem checking for devices : status', e.returncode, '\n')
    return match.group(1) if match else False


def killadb():
    try:
        check_output(['adb', 'kill-server'])
        print('Killed adb')
    except CalledProcessError as e:
        print('Problem stopping adb : status', e.returncode)


if __name__ == '__main__':
    atexit.register(killadb)
    run(host='127.0.0.1', port=8004)
