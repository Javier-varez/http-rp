addEventListener('load', (e) => {
    const ledSwitch = document.getElementById('ledSwitch')

    ledSwitch.addEventListener('click', async (e) => {
        if (ledSwitch.checked == true) {
            await fetch('/api/led/on', {method: 'POST'});
        } else {
            await fetch('/api/led/off', {method: 'POST'});
        }
    })
})
