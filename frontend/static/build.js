fetch('/health')
  .then((response) => response.ok ? response.json() : Promise.reject(new Error('health unavailable')))
  .then((health) => {
    for (const target of document.querySelectorAll('[data-build]')) {
      target.textContent = `Build ${String(health.build).slice(0, 12)}`;
    }
  })
  .catch(() => undefined);
