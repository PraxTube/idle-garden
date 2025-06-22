import './restart-audio-context.js'
import init from './bevy_game.js'

init().catch((error) => {
    if (!error.message.startsWith("Using exceptions for control flow, don't mind me. This isn't actually an error!")) {
        throw error;
    }
});

// Hide loading screen and move canvas when the game starts
const loading = document.getElementById('loading');
const observer = new MutationObserver(() => {
    const canvas = document.querySelector('canvas');
    if (canvas) {
        loading.style.display = 'none';
        console.log("removing loading");
        observer.disconnect();
    } else {
        console.warn("no canvas item");
    }
});
// Start observing the document for changes
observer.observe(document.body, { childList: true, subtree: true });

// Buffer game state
let bufferedGameState = {};
window.buffer_game_state = function(json) {
    try {
        bufferedGameState = JSON.parse(json);
    } catch (e) {
        console.error("Invalid state JSON:", e);
    }
};

// Save game before unloading the page
window.addEventListener('beforeunload', () => {
    for (const [key, value] of Object.entries(bufferedGameState)) {
        localStorage.setItem(key, value);
    }
});
