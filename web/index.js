let fidx = 0;

window.load_image = path => new Promise((res, rej) => {
    const image = document.createElement('img');
    image.src = `/assets${path}`;
    image.onload = () => res(image);
    image.onerror = err => rej(err);
});
window.load_font = async path => {
    const font = new FontFace(`loaded_font_${fidx++}`, `url(/assets${path})`);
    await font.load();
    document.fonts.add(font);
    return font;
};

import { init } from "./pkg/index.js";

const app = await init();

window.addEventListener('resize', () => {
    app.resize(window.innerWidth, window.innerHeight);
});

let previous = document.timeline.currentTime || performance.now();
function loop(time) {
    const delta = time - previous;
    app.tick(delta / 1000);
    requestAnimationFrame(loop);
}
requestAnimationFrame(loop);

