const socket = new WebSocket(`ws://${window.location.hostname}:${window.location.port}/live-reload`);

socket.addEventListener('open', () => {
    console.log('Live reload connected');
});

socket.addEventListener('error', (error) => {
    console.error('Live reload error:', error);
});

socket.addEventListener('message', (event) => {
    if (event.data === 'reload') {
        location.reload();
    }
});

socket.addEventListener('close', () => {
    console.log('Live reload connection lost. Reconnecting...');
    setTimeout(() => {
        location.reload();
    }, 1000);
}); 
