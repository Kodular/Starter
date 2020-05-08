
window.onscroll = function() {
    if (window.scrollY > 5)
        document.getElementsByClassName('d-header')[0].classList.add('with-shadow');
    else document.getElementsByClassName('d-header')[0].classList.remove('with-shadow')
}

function starterNotRunning() {
    document.getElementById("dialogStatus").innerHTML = "Starter is not running on your computer.<br>Check and try again";
    document.getElementById("dialogImage").src = "assets/images/starter-not-running.svg";
    dialog.open();
    snackbar.close()
}
checkrunning = document.getElementById("checkrunning"),
checkconnected = document.getElementById("checkconnected");
checkrunning.addEventListener("click", isRunning);
checkconnected.addEventListener("click", isConnected);

function isRunning() {
    snackbar.open();
    fetch("http://127.0.0.1:8004/ping/").then(res => res.json()).then(result => {
        if (result.status == "OK") {
            starterRunning()
        }
    }).catch(e => {
        starterNotRunning()
    })
}

function starterRunning() {
    document.getElementById("dialogImage").src = "assets/images/starter-running.svg";
    document.getElementById("dialogStatus").innerHTML = "Yes! Starter is running.<br>Now you can connect to Companion via USB";
    dialog.open();
    snackbar.close()
}

function phoneConnected() {
    document.getElementById("dialogImage").src = "assets/images/phone-connected.svg";
    document.getElementById("dialogStatus").innerHTML = "Your phone is connected!.<br>Now you can connect to Companion via USB";
    dialog.open();
    snackbar.close()
}

function phoneNoConnected() {
    document.getElementById("dialogImage").src = "assets/images/phone-not-connected.svg";
    document.getElementById("dialogStatus").innerHTML = "Starter is running, but has not found a device.<br> Please check and try again";
    dialog.open();
    snackbar.close()
}

function isConnected() {
    snackbar.open();
    resultado = "";
    fetch("http://127.0.0.1:8004/utest/").then(res => res.json()).then(result => {
        resultado = result;
        if (result.status == "OK") {
            phoneConnected()
        } else if (result.status == "NO") {
            phoneNoConnected()
        }
    }).catch(e => {
        if (resultado.status == "NO") {
            phoneNoConnected()
        } else starterNotRunning()
    })
}

function openUrl(url, external = !1) {
    window.open(url, external ? '_blank' : '_self')
}
const snackbar = new mdc.snackbar.MDCSnackbar(document.querySelector('.mdc-snackbar'));
const dialog = new mdc.dialog.MDCDialog(document.querySelector('.mdc-dialog'))
