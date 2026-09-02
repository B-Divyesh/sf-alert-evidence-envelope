<script lang="ts">
  import { onMount, tick } from 'svelte';

  type Config = {
    id: string; name: string; source_url: string; destination_url: string;
    destination_kind: string; query_pointer: string; evidence_pointer: string;
    service_pointer: string; error_pointer: string; time_pointer: string;
    redact_fields: string[]; max_items: number; max_bytes: number; enabled: boolean;
  };
  type Record = { id: string; service: string; status: string; fingerprint: string; created_at: string; evidence_items: number; evidence_bytes: number };
  type Preset = { name: string; fields: string[] };

  const slug = 'alert-evidence-envelope';
  const billingBase = 'https://api.sociobot.in/api/v1';
  const licenseKey = `sb_license:${slug}`;
  const verdictKey = `${licenseKey}:verdict`;
  const demoSessionKey = `demo:${slug}:session`;
  const demoPreviewKey = `demo:${slug}:preview`;
  const demoRouteKey = `demo:${slug}:route`;
  const demoRoutes = [
    { id: 'internal-slack', name: 'Internal Slack', fields: ['token'], destination: 'Slack incoming webhook' },
    { id: 'customer-automation', name: 'Customer automation', fields: ['email', 'token'], destination: 'JSON webhook' },
  ];
  const sampleAlert = `{
  "service": "checkout-api",
  "error": "payment authorization timed out",
  "startsAt": "2026-08-27T14:32:08Z",
  "query": "service=checkout-api level=error",
  "evidence": [
    {"timestamp":"2026-08-27T14:31:41Z","message":"gateway timeout after 8000ms","trace_id":"8af41b","email":"customer@example.com"},
    {"timestamp":"2026-08-27T14:31:58Z","message":"retry budget exhausted","token":"sk_live_secret"}
  ]
}`;

  let path = typeof location === 'undefined' ? '/' : location.pathname;
  let online = typeof navigator === 'undefined' ? true : navigator.onLine;
  let buildId = import.meta.env.VITE_BUILD_SHA || 'development';
  let config: Config = {
    id: 'primary', name: 'Primary incident route', source_url: '', destination_url: '', destination_kind: 'json',
    query_pointer: '/query', evidence_pointer: '/evidence', service_pointer: '/service', error_pointer: '/error',
    time_pointer: '/startsAt', redact_fields: ['authorization', 'password', 'token', 'email', 'cookie'],
    max_items: 20, max_bytes: 32768, enabled: true,
  };
  let channels: Config[] = [];
  let redactText = config.redact_fields.join(', ');
  let adminToken = '';
  let configState: 'locked' | 'loading' | 'ready' | 'saving' | 'saved' | 'error' = 'locked';
  let configMessage = 'Enter the server admin token to load this route.';
  let sample = sampleAlert;
  let preview: any = null;
  let previewState: 'idle' | 'loading' | 'success' | 'error' = 'idle';
  let previewMessage = '';
  let deliveries: Record[] = [];
  let copyMessage = '';
  let license = '';
  let licenseInput = '';
  let unlocked = false;
  let licenseMessage = 'Free core active';
  let presetName = '';
  let presets: Preset[] = [];
  let demoSession = '';
  let demoRoute = demoRoutes[1];
  let routeAnnouncement = '';

  onMount(() => {
    online = navigator.onLine;
    updateMetadata();
    if (online) void loadBuildIdentity();
    window.addEventListener('popstate', () => void setRoute(location.pathname, false));
    if (path === '/demo') {
      const savedRoute = localStorage.getItem(demoRouteKey);
      demoRoute = demoRoutes.find((route) => route.id === savedRoute) || demoRoutes[1];
      const cached = localStorage.getItem(demoPreviewKey);
      if (cached) {
        try {
          preview = JSON.parse(cached);
          previewState = 'success';
          previewMessage = online ? 'Sample ready. Starting a fresh demo…' : 'Offline sample ready. Demo data was not stored.';
        } catch { localStorage.removeItem(demoPreviewKey); }
      }
      void startDemo(false);
    } else {
      presets = JSON.parse(localStorage.getItem('envelope:presets') || '[]');
    }
    const fromUrl = new URL(location.href).searchParams.get('license');
    if (fromUrl) {
      localStorage.setItem(licenseKey, fromUrl);
      const clean = new URL(location.href); clean.searchParams.delete('license');
      window.history.replaceState({}, '', clean.pathname + clean.search + clean.hash);
    }
    license = localStorage.getItem(licenseKey) || '';
    void verifyLicense();
    window.addEventListener('online', updateOnline);
    window.addEventListener('offline', updateOnline);
    return () => {
      window.removeEventListener('popstate', () => void setRoute(location.pathname, false));
      window.removeEventListener('online', updateOnline);
      window.removeEventListener('offline', updateOnline);
    };
  });

  function updateOnline() {
    online = navigator.onLine;
    if (path === '/demo' && previewState === 'success') {
      previewMessage = online
        ? 'Envelope signed. Demo data was not stored.'
        : 'Offline sample ready. Demo data was not stored.';
    }
  }

  function updateMetadata() {
    const canonical = `https://alert-evidence-envelope.sociobot.in${path === '/demo' ? '/demo' : '/'}`;
    const title = path === '/demo' ? 'Demo — Alert Evidence Envelope' : 'Alert Evidence Envelope — add evidence to alerts';
    document.title = title;
    document.querySelector<HTMLLinkElement>('link[rel="canonical"]')?.setAttribute('href', canonical);
    document.querySelector<HTMLMetaElement>('meta[property="og:url"]')?.setAttribute('content', canonical);
    document.querySelector<HTMLMetaElement>('meta[property="og:title"]')?.setAttribute('content', title);
    document.querySelector<HTMLMetaElement>('meta[name="twitter:title"]')?.setAttribute('content', title);
  }

  async function setRoute(next: string, push = true) {
    if (next === path) return;
    if (push) history.pushState({}, '', next);
    path = next;
    updateMetadata();
    if (path === '/demo') await startDemo(false);
    await tick();
    const heading = document.querySelector<HTMLElement>('main h1');
    if (heading) { heading.tabIndex = -1; heading.focus(); routeAnnouncement = heading.textContent?.trim() || ''; }
    window.scrollTo(0, 0);
  }

  function navigate(event: MouseEvent, next: string, exitsDemo = false) {
    event.preventDefault();
    if (exitsDemo) leaveDemo();
    void setRoute(next);
  }

  async function loadBuildIdentity() {
    if (!navigator.onLine) return;
    try {
      const response = await fetch('/health');
      if (response.ok) buildId = (await response.json()).build || buildId;
    } catch { /* The cached shell remains usable offline. */ }
  }

  async function api(pathname: string, options: RequestInit = {}) {
    const headers = new Headers(options.headers);
    headers.set('content-type', 'application/json');
    if (adminToken) headers.set('authorization', `Bearer ${adminToken}`);
    const response = await fetch(pathname, { ...options, headers });
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.error || `Request failed with HTTP ${response.status}`);
    return body;
  }

  async function loadConfig() {
    if (!adminToken.trim()) {
      configState = 'error';
      configMessage = 'Enter the admin token stored on the relay host.';
      return;
    }
    configState = 'loading';
    try {
      const loaded = await api('/api/v1/config');
      config = { ...loaded, source_url: loaded.source_url || '', destination_url: loaded.destination_url || '' };
      channels = await api('/api/v1/channels');
      redactText = config.redact_fields.join(', ');
      configState = 'ready'; configMessage = 'Route loaded from this relay';
      deliveries = await api('/api/v1/history');
    } catch (error) {
      configState = 'error';
      configMessage = error instanceof Error ? error.message : 'Could not reach the relay';
    }
  }

  async function selectRoute(id: string) {
    try {
      const loaded = await api(`/api/v1/channels/${id}`);
      config = { ...loaded, source_url: loaded.source_url || '', destination_url: loaded.destination_url || '' };
      redactText = config.redact_fields.join(', ');
      configMessage = `Editing ${config.name}. Each route has its own inbound URL and redaction list.`;
    } catch (error) { configMessage = error instanceof Error ? error.message : 'Could not load the route'; }
  }

  async function createRoute() {
    try {
      const created = await api('/api/v1/channels', { method: 'POST', body: JSON.stringify({ ...config, id: '', name: 'New delivery route' }) });
      channels = [...channels, created];
      await selectRoute(created.id);
    } catch (error) { configMessage = error instanceof Error ? error.message : 'Could not create a route'; }
  }

  async function deleteRoute() {
    if (config.id === 'primary') return;
    try {
      await api(`/api/v1/channels/${config.id}`, { method: 'DELETE' });
      channels = channels.filter((channel) => channel.id !== config.id);
      await selectRoute('primary');
    } catch (error) { configMessage = error instanceof Error ? error.message : 'Could not delete the route'; }
  }

  async function saveConfig(event: SubmitEvent) {
    event.preventDefault(); configState = 'saving'; configMessage = 'Checking and saving the route…';
    const outgoing = {
      ...config,
      source_url: config.source_url.trim() || null,
      destination_url: config.destination_url.trim() || null,
      redact_fields: redactText.split(',').map((v) => v.trim()).filter(Boolean),
    };
    try {
      const endpoint = config.id === 'primary' ? '/api/v1/config' : `/api/v1/channels/${config.id}`;
      const saved = await api(endpoint, { method: 'PUT', body: JSON.stringify(outgoing) });
      config = { ...saved, source_url: saved.source_url || '', destination_url: saved.destination_url || '' };
      channels = channels.map((channel) => channel.id === saved.id ? saved : channel);
      redactText = config.redact_fields.join(', ');
      configState = 'saved'; configMessage = 'Route saved. New alerts use this policy.';
    } catch (error) {
      configState = 'error'; configMessage = error instanceof Error ? error.message : 'The route could not be saved';
    }
  }

  async function runPreview() {
    if (!navigator.onLine) {
      online = false;
      if (path === '/demo' && preview) {
        previewState = 'success';
        previewMessage = 'Offline sample ready. Demo data was not stored.';
      } else {
        previewState = 'error';
        previewMessage = 'The relay is offline. Reconnect before building a preview.';
      }
      return;
    }
    previewState = 'loading'; previewMessage = 'Bounding and redacting evidence…'; preview = null;
    try {
      const alert = JSON.parse(sample);
      const endpoint = path === '/demo'
        ? `/api/v1/demo/sessions/${demoSession}/preview`
        : '/api/v1/preview';
      preview = await api(endpoint, {
        method: 'POST', body: JSON.stringify({
          alert,
          redact_fields: path === '/demo' ? demoRoute.fields : redactText.split(',').map((v) => v.trim()).filter(Boolean),
          max_items: config.max_items, max_bytes: config.max_bytes,
        }),
      });
      if (path === '/demo') localStorage.setItem(demoPreviewKey, JSON.stringify(preview));
      previewState = 'success';
      previewMessage = path === '/demo'
        ? 'Envelope signed. Demo data was not stored.'
        : 'Envelope signed. Preview data was not stored.';
    } catch (error) {
      previewState = 'error';
      previewMessage = error instanceof SyntaxError ? 'Sample alert is not valid JSON. Check commas and quotes.' : (error instanceof Error ? error.message : 'Preview failed');
    }
  }

  async function startDemo(reset: boolean) {
    if (!navigator.onLine) {
      online = false;
      if (!preview) {
        previewState = 'error';
        previewMessage = 'The sample is not cached yet. Reconnect once to start the demo.';
      }
      return;
    }
    const previous = localStorage.getItem(demoSessionKey) || '';
    if (reset && previous) {
      await fetch(`/api/v1/demo/sessions/${previous}`, { method: 'DELETE' }).catch(() => undefined);
      localStorage.removeItem(demoPreviewKey);
    }
    sample = sampleAlert;
    preview = null;
    previewState = 'loading';
    previewMessage = 'Starting an isolated sample workspace…';
    try {
      const response = await api('/api/v1/demo/sessions', { method: 'POST', body: '{}' });
      demoSession = response.id;
      localStorage.setItem(demoSessionKey, demoSession);
      await runPreview();
    } catch (error) {
      previewState = 'error';
      previewMessage = error instanceof Error ? error.message : 'The sample could not start.';
    }
  }

  async function selectDemoRoute(id: string) {
    demoRoute = demoRoutes.find((route) => route.id === id) || demoRoutes[0];
    localStorage.setItem(demoRouteKey, demoRoute.id);
    await runPreview();
  }

  function leaveDemo() {
    const session = localStorage.getItem(demoSessionKey);
    if (session && online) void fetch(`/api/v1/demo/sessions/${session}`, { method: 'DELETE' });
    localStorage.removeItem(demoSessionKey);
    localStorage.removeItem(demoPreviewKey);
    localStorage.removeItem(demoRouteKey);
  }

  async function copy(value: string, message: string) {
    try { await navigator.clipboard.writeText(value); copyMessage = message; }
    catch { copyMessage = 'Clipboard access is unavailable. Select and copy the text manually.'; }
  }

  async function verifyLicense(force = false) {
    license = localStorage.getItem(licenseKey) || '';
    const cached = JSON.parse(localStorage.getItem(verdictKey) || 'null') as { valid: boolean; checkedAt?: number; attemptedAt?: number } | null;
    if (cached?.valid) { unlocked = true; licenseMessage = 'Field Kit unlocked'; }
    if (!license) return;
    if (!force && cached && Date.now() - (cached.attemptedAt || cached.checkedAt || 0) < 86_400_000) {
      unlocked = cached.valid; return;
    }
    try {
      const attemptedAt = Date.now();
      localStorage.setItem(verdictKey, JSON.stringify({ valid: cached?.valid === true, checkedAt: cached?.checkedAt, attemptedAt }));
      const response = await fetch(`${billingBase}/products/${slug}/verify`, { headers: { authorization: `Bearer ${license}` } });
      const result = await response.json();
      unlocked = result.valid === true;
      localStorage.setItem(verdictKey, JSON.stringify({ valid: unlocked, checkedAt: attemptedAt, attemptedAt }));
      licenseMessage = unlocked ? 'Field Kit unlocked' : 'License no longer active';
    } catch {
      licenseMessage = unlocked ? 'Field Kit unlocked · verification pending' : 'Could not verify while offline';
    }
  }

  async function restoreLicense(event: SubmitEvent) {
    event.preventDefault();
    if (!licenseInput.trim()) { licenseMessage = 'Paste a license token first'; return; }
    localStorage.setItem(licenseKey, licenseInput.trim()); localStorage.removeItem(verdictKey);
    licenseInput = ''; await verifyLicense(true);
  }

  function savePreset() {
    if (!unlocked || !presetName.trim()) return;
    presets = [...presets.filter((p) => p.name !== presetName.trim()), { name: presetName.trim(), fields: redactText.split(',').map((v) => v.trim()).filter(Boolean) }];
    localStorage.setItem('envelope:presets', JSON.stringify(presets)); presetName = '';
  }

  function applyPreset(preset: Preset) { redactText = preset.fields.join(', '); configMessage = `Applied “${preset.name}”. Save the route to activate it.`; }
  function formatBytes(bytes: number) { return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(1)} KB`; }
  function formatDate(value: string) { const date = new Date(value); return Number.isNaN(date.getTime()) ? value : date.toLocaleString(); }
</script>

<svelte:head>
  <title>{path === '/privacy' ? 'Privacy — Alert Evidence Envelope' : path === '/terms' ? 'Terms — Alert Evidence Envelope' : path === '/demo' ? 'Demo — Alert Evidence Envelope' : 'Alert Evidence Envelope — add evidence to alerts'}</title>
</svelte:head>

<p class="sr-status" aria-live="polite">{routeAnnouncement}</p>

<a class="skip-link" href="#main">Skip to main content</a>
<header class="site-header">
  <a class="brand" href="/" aria-label="Alert Evidence Envelope home">
    <svg aria-hidden="true" viewBox="0 0 48 48"><path d="M7 12h34v24H7z"/><path d="m8 14 16 12 16-12M14 8c4-4 16-4 20 0"/></svg>
    <span>Alert Evidence<br /><em>Envelope</em></span>
  </a>
  <nav aria-label="Primary navigation">
    {#if path === '/'}
    <a href="/demo" onclick={(event) => navigate(event, '/demo')}>Demo</a><a href="#configure">Configure</a><a href="/privacy">Privacy</a>
    {:else if path === '/demo'}<a href="/" onclick={(event) => navigate(event, '/', true)}>Start for real</a><a href="/privacy">Privacy</a>
    {:else}<a href="/">Home</a><a href="/demo">Demo</a><a href="#main" aria-current="page">{path === '/privacy' ? 'Privacy' : 'Terms'}</a>{/if}
  </nav>
  <span class:offline={!online} class="network"><i></i>{online ? 'Browser online' : 'Browser offline'}</span>
</header>

{#if path === '/demo'}
  <aside class="demo-banner" aria-label="Demo status">
    <strong>Demo — sample data, nothing is saved</strong>
    <span>Isolated workspace expires after 24 hours.</span>
    <button type="button" onclick={() => startDemo(true)}>Reset demo</button>
    <a href="/" onclick={(event) => navigate(event, '/', true)}>Start for real</a>
  </aside>
{/if}

<main id="main">
{#if path === '/privacy'}
  <article class="legal">
    <p class="eyebrow">Privacy notice</p><h1>How this relay handles data</h1>
    <p class="lede">The self-hosted core is designed to transform incident data without retaining raw alert bodies or raw fetched logs.</p>
    <h2>What the relay stores</h2><p>SQLite stores channel settings, short-lived demo session IDs, and a 20-entry delivery ledger. Demo session rows contain only an ID and expiry time. The relay does not store raw alerts, evidence, or license tokens.</p>
    <h2>Where secrets live</h2><p>Upstream and destination bearer tokens, admin access, and the signing key come from environment variables. Destination and source URLs may be stored in the local SQLite configuration. Browser license tokens and paid policy presets remain in your browser’s local storage.</p>
    <h2>Network requests</h2><p>The relay contacts only endpoints you configure. License verification contacts Sociobot when a license is present, at most once per day. There are no analytics, advertising cookies, third-party scripts, or hosted fonts.</p>
    <h2>Control and deletion</h2><p>The operator controls the SQLite database and browser storage. Remove the database or clear site data to delete them.</p>
    <p class="updated">Effective 27 August 2026</p>
  </article>
{:else if path === '/terms'}
  <article class="legal">
    <p class="eyebrow">Terms</p><h1>Terms of use</h1>
    <p class="lede">Alert Evidence Envelope is a transformation and delivery tool. It does not evaluate alerts, replace your incident system, or guarantee delivery.</p>
    <h2>Operator responsibility</h2><p>You are responsible for endpoint authorization, lawful processing, redaction policies, destination access, secret rotation, and testing size limits before production use. Do not place credentials in JSON payloads or browser configuration.</p>
    <h2>Field Kit license</h2><p>The $39 Field Kit is a one-time license for reusable local policy presets. Redaction, signing, previews, copying, and route settings remain free.</p>
    <h2>Warranty and liability</h2><p>The software is provided “as is,” without warranties. To the extent permitted by law, contributors are not liable for lost data, missed notifications, or indirect damages. Validate the relay in your own environment and retain your source system as the record of truth.</p>
    <h2>Acceptable use</h2><p>Do not use the service to access systems without permission, evade provider controls, or transmit data prohibited by your organization or applicable law.</p>
    <p class="updated">Effective 27 August 2026</p>
  </article>
{:else}
{#if path === '/'}
  <section class="hero" aria-labelledby="hero-title">
    <div class="hero-copy">
      <p class="eyebrow">Add evidence to webhook alerts</p>
      <h1 id="hero-title">Add redacted evidence to webhook alerts</h1>
      <p class="lede">For on-call engineers and webhook consumers who need incident context without another dashboard login.</p>
      <div class="hero-actions"><a class="button primary" href="/demo">Try it with sample data</a><a class="button secondary" href="#configure">Configure your route</a></div>
      <p class="action-note">The sample opens a signed, redacted envelope in an isolated workspace.</p>
      <ul class="trust-list" aria-label="Product facts"><li>Demo data is never added to route history</li><li>No analytics or third-party scripts</li><li>Self-hosted core is free; Field Kit costs $39 once</li></ul>
    </div>
    <figure class="terrain">
      <picture><source media="(max-width: 700px)" srcset="/assets/evidence-terrain-960.webp" /><img src="/assets/evidence-terrain-1536.webp" width="1536" height="1024" decoding="async" fetchpriority="high" alt="An amber alert path crosses a topographic incident map, passes a redaction mark, and arrives at a sealed green envelope." /></picture>
      <figcaption><span>BOUNDARY 32 KB</span><span>REDACT → SIGN</span></figcaption>
    </figure>
  </section>

  <section class="route" aria-labelledby="route-title">
    <div class="section-heading"><p class="eyebrow">How it works</p><h2 id="route-title">Four checks before delivery</h2></div>
    <ol class="route-stages">
      <li><span>01</span><h3>Limit the evidence</h3><p>Use one fixed source. Limit the record count and envelope size.</p></li>
      <li><span>02</span><h3>Remove sensitive fields</h3><p>Remove sensitive keys recursively with this route’s redaction list before forwarding.</p></li>
      <li><span>03</span><h3>Record the source and query</h3><p>Hash the configured query and source so responders know what shaped the excerpt.</p></li>
      <li><span>04</span><h3>Sign the envelope</h3><p>Sign the final JSON envelope and preserve the provider signature in transit when present.</p></li>
    </ol>
  </section>

  <section id="configure" class="workspace" aria-labelledby="configure-title">
    <div class="workspace-intro"><p class="eyebrow">Protected route settings</p><h2 id="configure-title">Configure delivery routes</h2><p>Each route stores its own redaction list and destination. Server credentials stay outside the browser.</p></div>
    <div class="state-strip" class:error={configState === 'error'} class:success={configState === 'saved'} aria-live="polite"><span></span>{configMessage}</div>
    {#if channels.length}<div class="route-list" aria-label="Delivery routes">{#each channels as channel}<button class:active={channel.id === config.id} type="button" onclick={() => selectRoute(channel.id)}>{channel.name}<small>{channel.id}</small></button>{/each}<button type="button" onclick={createRoute}>Create route</button>{#if config.id !== 'primary'}<button type="button" onclick={deleteRoute}>Delete this route</button>{/if}</div>{/if}
    <form onsubmit={saveConfig}>
      <fieldset><legend><b>1</b> Name and enable the route</legend>
        <div class="form-grid"><label>Route name<input bind:value={config.name} required maxlength="80" /></label><label class="toggle"><input type="checkbox" bind:checked={config.enabled} /><span>Accept incoming alerts</span></label></div>
      </fieldset>
      <fieldset><legend><b>2</b> Choose the evidence source</legend>
        <p class="field-help">Leave the source blank when evidence already arrives inside the alert. A remote source receives only <code>?q=…&amp;limit=…</code>.</p>
        <div class="form-grid"><label>Fixed evidence source URL <span>optional</span><input type="url" bind:value={config.source_url} placeholder="https://logs.internal.example/query" aria-describedby="source-help" /></label><label>Query JSON pointer<input bind:value={config.query_pointer} required pattern="/.*" aria-describedby="query-help" /><small id="query-help">Path to the query field, for example <code>/query</code>.</small></label><label>Embedded evidence pointer<input bind:value={config.evidence_pointer} required pattern="/.*" /></label><label>Upstream token<input value="UPSTREAM_BEARER_TOKEN" disabled /><small id="source-help">Set in the server environment; never entered here.</small></label></div>
      </fieldset>
      <fieldset><legend><b>3</b> Limit and redact evidence</legend>
        <div class="form-grid"><label>Redact keys <span>comma-separated</span><textarea bind:value={redactText} rows="3" required></textarea></label><div class="split"><label>Maximum records<input type="number" bind:value={config.max_items} min="1" max="100" required /></label><label>Maximum envelope bytes<input type="number" bind:value={config.max_bytes} min="1024" max="131072" step="1024" required /></label></div></div>
      </fieldset>
      <fieldset><legend><b>4</b> Choose the delivery destination</legend>
        <div class="form-grid"><label>Destination type<select bind:value={config.destination_kind}><option value="json">JSON webhook</option><option value="slack">Slack incoming webhook</option><option value="email-webhook">Email gateway webhook</option></select><small>JSON receives the envelope. Slack adds a readable <code>text</code> field. Email gateways receive <code>subject</code>, <code>text</code>, and <code>envelope</code>.</small></label><label>Destination URL <span>optional if set by environment</span><input type="url" bind:value={config.destination_url} placeholder="https://hooks.example/…" /></label><label>Service JSON pointer<input bind:value={config.service_pointer} required pattern="/.*" /></label><label>Error JSON pointer<input bind:value={config.error_pointer} required pattern="/.*" /></label><label>First-seen JSON pointer<input bind:value={config.time_pointer} required pattern="/.*" /></label><label>Admin token <span>read from the relay host</span><input type="password" bind:value={adminToken} autocomplete="off" /></label></div>
        <button class="copy load-route" type="button" onclick={loadConfig} disabled={configState === 'loading'}>{configState === 'loading' ? 'Loading route…' : 'Load protected route'}</button>
      </fieldset>
      <div class="form-actions"><button class="button primary" type="submit" disabled={configState === 'saving'}>{configState === 'saving' ? 'Saving route…' : 'Save route'}</button><code>POST {typeof location === 'undefined' ? '' : location.origin}/api/v1/relay/{config.id}</code><button class="copy" type="button" onclick={() => copy(`${location.origin}/api/v1/relay/${config.id}`, 'Relay URL copied')}>Copy relay URL</button><small>Incoming requests must send the server’s <code>x-envelope-token</code>.</small></div>
    </form>
    <p class="sr-status" aria-live="polite">{copyMessage}</p>
  </section>
{/if}

  <section id="test" class="test-bench" aria-labelledby="test-title">
    <div class="section-heading"><p class="eyebrow">Envelope preview</p>{#if path === '/demo'}<h1 id="test-title">Inspect a sample evidence envelope</h1><p>The sample runs automatically in an isolated workspace. It never changes the protected route.</p>{:else}<h2 id="test-title">Inspect an envelope before delivery</h2><p>Preview applies the live route’s bounds, redaction, fingerprint, and signature. It does not add delivery history.</p>{/if}</div>
    <div class="bench-grid">
      <div>{#if path === '/demo'}<div class="demo-routes" aria-label="Sample routes">{#each demoRoutes as route}<button type="button" class:active={demoRoute.id === route.id} onclick={() => selectDemoRoute(route.id)}><b>{route.name}</b><span>{route.destination} · removes {route.fields.join(' and ')}</span></button>{/each}</div>{/if}<label for="sample-json">Sample alert JSON</label><textarea id="sample-json" class="code-area" bind:value={sample} spellcheck="false"></textarea><button class="button amber" type="button" onclick={runPreview} disabled={previewState === 'loading'}>{previewState === 'loading' ? 'Sealing…' : 'Build signed preview'}</button></div>
      <div class="envelope-output" class:demo-output={path === '/demo'} aria-busy={previewState === 'loading'}>
        {#if previewState === 'idle'}<div class="empty"><svg aria-hidden="true" viewBox="0 0 64 64"><path d="M9 18h46v32H9zM10 20l22 17 22-17"/><path d="M24 12c5-5 11-5 16 0"/></svg><h3>No envelope yet</h3><p>Use the sample as-is or paste a realistic alert with sensitive values removed.</p></div>
        {:else if previewState === 'loading'}<div class="empty"><div class="loader" aria-hidden="true"></div><h3>Building the envelope</h3><p>Bounding → redacting → fingerprinting → signing</p></div>
        {:else if previewState === 'error'}<div class="empty error-panel"><b>Preview stopped</b><p>{previewMessage}</p><button type="button" onclick={() => sample = sampleAlert}>Restore valid sample</button></div>
        {:else}
          <div class="envelope-head"><span>SEALED</span><code>{preview.schema}</code></div>
          {#if path === '/demo'}<p class="demo-sealed" aria-live="polite">{previewMessage}</p>{/if}
          <p class="redaction-result"><b>Sensitive fields</b> [REDACTED]</p>
          {#if path === '/demo'}<p class="demo-route-result"><b>{demoRoute.name}</b> removes {preview.redacted_fields.join(', ')} before delivery.</p>{/if}
          <dl class="summary"><div><dt>Service</dt><dd>{preview.summary.service}</dd></div><div><dt>Error signature</dt><dd>{preview.summary.error_signature}</dd></div><div><dt>First seen</dt><dd>{formatDate(preview.summary.first_seen)}</dd></div></dl>
          <div class="coordinates"><span><b>{preview.evidence_items}</b> items</span><span><b>{formatBytes(preview.evidence_bytes)}</b> evidence</span><span><b>{preview.truncated ? 'Yes' : 'No'}</b> truncated</span></div>
          <p class="fingerprint"><span>Query fingerprint</span><code>{preview.query_fingerprint}</code></p>
          <details><summary>Inspect signed JSON</summary><!-- svelte-ignore a11y_no_noninteractive_tabindex (the bounded scroll region must accept keyboard focus) --><pre tabindex="0" aria-label="Signed evidence envelope JSON">{JSON.stringify(preview, null, 2)}</pre></details>
          <button class="copy" type="button" onclick={() => copy(JSON.stringify(preview, null, 2), 'Signed envelope copied')}>Copy envelope JSON</button>
          <p class="copy-feedback" aria-live="polite">{copyMessage}</p>
        {/if}
        {#if !(path === '/demo' && previewState === 'success')}<p class:bad={previewState === 'error'} class="bench-status" aria-live="polite">{previewMessage}</p>{/if}
      </div>
    </div>
  </section>

{#if path === '/'}
  <section class="ledger" aria-labelledby="ledger-title">
    <div class="section-heading"><p class="eyebrow">Last 20 deliveries</p><h2 id="ledger-title">Recent delivery metadata</h2><p>Delivery history stores metadata only. Raw alerts and evidence are absent.</p></div>
    {#if deliveries.length}
      <div class="table-wrap"><table><thead><tr><th>Created</th><th>Service</th><th>Status</th><th>Evidence</th><th>Fingerprint</th></tr></thead><tbody>{#each deliveries as item}<tr><td data-label="Created">{formatDate(item.created_at)}</td><td data-label="Service">{item.service}</td><td data-label="Status"><span class="status-dot"></span>{item.status}</td><td data-label="Evidence">{item.evidence_items} · {formatBytes(item.evidence_bytes)}</td><td data-label="Fingerprint"><code>{item.fingerprint}</code></td></tr>{/each}</tbody></table></div>
    {:else}<div class="ledger-empty"><span>∅</span><div><h3>No delivery metadata yet</h3><p>Send a live alert to the relay URL. Preview runs never appear here.</p></div></div>{/if}
  </section>

  <section id="field-kit" class="field-kit" aria-labelledby="kit-title">
    <div><p class="eyebrow">Optional local presets</p><h2 id="kit-title">Reuse redaction policies</h2><p>Redaction, signing, previews, copying, and route settings are free. The <strong>$39 Field Kit</strong> is a one-time purchase.</p><p>It adds named redaction presets on this device.</p><ul><li>Named policies for Slack, customers, and automation</li><li>Apply a policy before saving a route</li><li>Checkout is hosted by Sociobot</li></ul></div>
    <div class="license-card" class:unlocked>
      <span class="license-state">{unlocked ? '✓ LICENSE ACTIVE' : 'FIELD KIT · $39 ONCE'}</span>
      {#if unlocked}
        <label>Preset name<input bind:value={presetName} maxlength="40" placeholder="Customer-facing Slack" /></label><button class="button primary" type="button" onclick={savePreset}>Save current redaction policy</button>
        {#if presets.length}<ul class="presets">{#each presets as preset}<li><button type="button" onclick={() => applyPreset(preset)}><b>{preset.name}</b><span>{preset.fields.join(', ')}</span></button></li>{/each}</ul>{:else}<p class="quiet">No presets yet. Name the current policy to keep it locally.</p>{/if}
      {:else}
        <a class="button primary" href={`${billingBase}/products/${slug}/checkout`}>Buy the Field Kit</a>
        <form class="restore" onsubmit={restoreLicense}><label for="license-token">Have a license? Paste it</label><div><input id="license-token" type="password" bind:value={licenseInput} autocomplete="off" /><button type="submit">Verify license</button></div></form>
      {/if}
      <p class="license-message" aria-live="polite">{licenseMessage}</p>
    </div>
  </section>
{/if}
{/if}
</main>

<footer>
  <div><a class="brand footer-brand" href="/"><svg aria-hidden="true" viewBox="0 0 48 48"><path d="M7 12h34v24H7z"/><path d="m8 14 16 12 16-12"/></svg><span>Alert Evidence Envelope</span></a><p>Send bounded incident evidence with a webhook alert.</p></div>
  <div class="footer-links"><a href="/privacy">Privacy</a><a href="/terms">Terms</a><a href="https://github.com/B-Divyesh/sf-alert-evidence-envelope">Source (external)</a></div>
  <p class="provenance">Built by Param Factory · Build {buildId.slice(0, 12)} · Cartography generated for this product on 2026-08-27 · MIT licensed</p>
</footer>
