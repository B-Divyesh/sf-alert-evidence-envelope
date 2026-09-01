const CACHE = 'envelope-shell-v5';
const SHELL = ['/', '/demo', '/privacy', '/terms', '/404.html', '/legal.css', '/build.js', '/favicon.svg', '/apple-touch-icon.png', '/fonts/inter-latin.woff2', '/fonts/fraunces-latin.woff2', '/assets/evidence-terrain-960.webp'];

self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(CACHE).then(async (cache) => {
    await Promise.all(SHELL.map((url) => cache.add(url).catch(() => undefined)));
    const shell = await cache.match('/');
    const html = shell ? await shell.text() : '';
    const builtAssets = [...html.matchAll(/(?:src|href)="(\/assets\/[^\"]+)"/g)].map((match) => match[1]);
    await Promise.all([...new Set(builtAssets)].map((url) => cache.add(url).catch(() => undefined)));
  }).then(() => self.skipWaiting()));
});

self.addEventListener('activate', (event) => {
  event.waitUntil(caches.keys().then((keys) => Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key)))).then(() => self.clients.claim()));
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  const url = new URL(request.url);
  if (request.method !== 'GET' || url.origin !== self.location.origin || url.pathname.startsWith('/api/') || url.pathname === '/health') return;
  if (request.mode === 'navigate') {
    event.respondWith(caches.match(request).then(async (cached) => {
      if (!self.navigator.onLine && cached) return cached;
      try {
        const response = await fetch(request);
        const copy = response.clone();
        void caches.open(CACHE).then((cache) => cache.put(request, copy));
        return response;
      } catch {
        return cached || caches.match('/');
      }
    }));
    return;
  }
  event.respondWith(caches.match(request).then((cached) => cached || fetch(request).then((response) => {
    const copy = response.clone(); caches.open(CACHE).then((cache) => cache.put(request, copy)); return response;
  })));
});
