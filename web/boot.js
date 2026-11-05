// Loads the WebAssembly app. Kept as an external file so the CSP needs no 'unsafe-inline'.
import init from './fit_web.js';

init().catch((err) => {
  const el = document.getElementById('loading');
  if (el) el.textContent = 'Failed to load WebAssembly: ' + err;
});
