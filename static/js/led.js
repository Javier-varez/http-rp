addEventListener('load', (e) => {
    const ledSwitch = document.getElementById('ledSwitch')

    ledSwitch.addEventListener('click', async (e) => {
        if (ledSwitch.checked == true) {
            await fetch('/led/on', {method: 'POST'});
        } else {
            await fetch('/led/off', {method: 'POST'});
        }
    })
})
