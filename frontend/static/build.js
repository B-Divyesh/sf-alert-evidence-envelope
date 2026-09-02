const routeFocusKey = 'alert-evidence-envelope:route-focus-pending';
const routeAnnouncer = document.querySelector('[data-route-announcer]');
const routeHeading = document.querySelector('main h1');

function markCrossDocumentNavigation() {
  sessionStorage.setItem(routeFocusKey, 'true');
}

function restoreCrossDocumentFocus(event) {
  const pending = sessionStorage.getItem(routeFocusKey) === 'true';
  if (!pending && !event?.persisted) return;
  sessionStorage.removeItem(routeFocusKey);
  requestAnimationFrame(() => {
    if (routeHeading instanceof HTMLElement) {
      routeHeading.focus();
      window.scrollTo(0, 0);
    }
    if (routeAnnouncer) routeAnnouncer.textContent = document.title;
  });
}

window.addEventListener('pagehide', markCrossDocumentNavigation);
window.addEventListener('pageshow', restoreCrossDocumentFocus);
restoreCrossDocumentFocus();

fetch('/health')
  .then((response) => response.ok ? response.json() : Promise.reject(new Error('health unavailable')))
  .then((health) => {
    for (const target of document.querySelectorAll('[data-build]')) {
      target.textContent = `Build ${String(health.build).slice(0, 12)}`;
    }
  })
  .catch(() => undefined);
