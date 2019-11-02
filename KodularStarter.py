#!/usr/bin/env python
import atexit
import platform
import re
from subprocess import check_call, check_output, CalledProcessError

from bottle import run, route, response

from utils import get_adb_exe

VERSION = '1'
PACKAGE_NAME = 'io.makeroid.companion'
ADB = get_adb_exe()


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
    device = get_device()
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
    device = get_device()
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
    kill_adb()
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
            [ADB,
             '-s', device,
             'forward', 'tcp:8001', 'tcp:8001'])
        check_call(
            [ADB,
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


def get_device():
    print('Finding device...')
    try:
        result = check_output([ADB, 'devices'])
        lines = result.decode('utf-8').strip().splitlines()
        for line in lines[1:]:
            if not line or line.startswith('*') or 'offline' in line:
                continue
            if re.search(r'(emulator-\d+)\s+device', line):  # We are an emulator
                continue  # Skip it
            match = re.search(r'(\w+)\s+device', line)
            if match:
                return match.group(1)
    except CalledProcessError as e:
        print('Problem checking for devices : status', e.returncode, '\n')
    return None


def kill_adb():
    try:
        check_output([ADB, 'kill-server'])
        print('Killed adb')
    except CalledProcessError as e:
        print('Problem stopping adb : status', e.returncode, '\n')


if __name__ == '__main__':
    print('Kodular Starter version:', VERSION)
    print('OS:', platform.system())
    print('Architecture:', platform.architecture()[0])
    print('Machine:', platform.machine())
    print('ADB path:', ADB)
    print('- ' * 31)

    atexit.register(kill_adb)

    run(host='127.0.0.1', port=8004)  # Run Bottle server
