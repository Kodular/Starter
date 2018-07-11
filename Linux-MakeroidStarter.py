#!/usr/bin/python
from bottle import run, route, app, request, response, template, default_app, Bottle, debug, abort
import sys
import os
import platform
import subprocess
import re

app = Bottle()
default_app.push(app)

def dash():
    print("_ "*30 + "\n")

VERSION = "1.0.0"
print("Makeroid Starter Andromeda")
dash()

platforms = platform.uname()[0]
print("OS = {}".format(platforms))
if platforms == 'Windows' or platforms == "Linux":
    if getattr(sys, 'frozen', False):
        # frozen
        PLATDIR = '"' + os.path.dirname(sys.executable) + '"'
    else:
        # unfrozen
        PLATDIR = os.path.dirname(os.path.realpath(__file__))
    print("Installation Path: {}".format(PLATDIR))
else:
    sys.exit(1)
dash()

@app.route('/ping/')
def ping():
    print("Ping...")
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    return '{ "status" : "OK", "version" : "%s" }' % VERSION

@app.route('/utest/')
def utest():
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    device = checkrunning(False)
    if device:
        return '{ "status" : "OK", "device" : "%s", "version" : "%s" }' % (device, VERSION)
    else:
        return '{ "status" : "NO", "version" : "%s" }' % VERSION

@app.route('/start/')
def start():
    subprocess.call(PLATDIR + "\\Makeroid\\run-emulator ", shell=True)
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    return ''

@app.route('/emulatorreset/')
def emulatorreset():
    subprocess.call(PLATDIR + "\\Makeroid\\reset-emulator ", shell=True)
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    return ''

@app.route('/echeck/')
def echeck():
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    device = checkrunning(True)
    if device:
        return '{ "status" : "OK", "device" : "%s", "version" : "%s"}' % (device, VERSION)
    else:
        return '{ "status" : "NO", "version" : "%s" }' % VERSION

@app.route('/ucheck/')
def ucheck():
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    device = checkrunning(False)
    if device:
        return '{ "status" : "OK", "device" : "%s", "version" : "%s"}' % (device, VERSION)
    else:
        return '{ "status" : "NO", "version" : "%s" }' % VERSION

@app.route('/reset/')
def reset():
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
    response.headers['Content-Type'] = 'application/json'
    killadb()
    killemulator()
    print("Reset done...")
    return '{ "status" : "OK", "version" : "%s" }' % VERSION

@app.route('/replstart/:device')
def replstart(device=None):
    print("Device = %s" % device)
    try:
        subprocess.check_output((PLATDIR + "\\adb.exe -s %s forward tcp:8001 tcp:8001") % device, shell=True)
        if re.match('.*emulat.*', device): #  Only fake the menu key for the emulator
            subprocess.check_output((PLATDIR + "\\adb -s %s shell input keyevent 82") % device, shell=True)
        subprocess.check_output((PLATDIR + "\\adb -s %s shell am start -a android.intent.action.VIEW -n io.makeroid.companion/.Screen1 --ez rundirect true") % device, shell=True)
        response.headers['Access-Control-Allow-Origin'] = '*'
        response.headers['Access-Control-Allow-Headers'] = 'origin, content-type'
        return ''
    except subprocess.CalledProcessError as e:
        print("Problem starting companion app : status %i\n" % e.returncode)
        return ''


def checkrunning(emulator):
    print("Checking device...")
    try:
        result = subprocess.check_output(PLATDIR + "\\adb.exe devices", shell=True)
        lines = str(result).split("\\r\\n")
        for line in lines[1:]:
            if emulator:
                m = re.search("^(emulator-[1-9]+)(\\+t)(device)", line)
            else:
                if re.search("^(emulator-[1-9]+)(\\+t)(device)", line): # We are an emulator
                    continue
                m = re.search(r"^([a-zA-Z0-9]+)(\\*t)(device)", line)
            if m:
                break
        if m:
            return m.group(1)
        return False
    except subprocess.CalledProcessError as e:
        print("Problem checking for devices : status %i\n" % e.returncode)
        return False

def killadb():
    try:
        subprocess.check_output(PLATDIR + "\\adb.exe kill-server", shell=True)
        print("Killed adb\n")
    except subprocess.CalledProcessError as e:
        print("Problem stopping adb : status %i\n" % e.returncode)
        return ''

def killemulator():
    try:
        subprocess.check_output(PLATDIR + "\\Makeroid\\kill-emulator", shell=True)
        print("Killed emulator\n")
    except subprocess.CalledProcessError as e:
        print("Problem stopping emulator : status %i\n" % e.returncode)
        return ''

def shutdown():
    try:                                # Be quiet...
        killadb()
        killemulator()
    except:
        pass

if __name__ == '__main__':
    import atexit
    atexit.register(shutdown)
    run(app, host='127.0.0.1', port=8004)
