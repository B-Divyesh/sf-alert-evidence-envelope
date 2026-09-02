# Visual thesis — incident topographic cartography

Alert Evidence Envelope treats an incident as terrain that must be surveyed before it can be safely handed off. Contour lines mean bounded scope; coordinate marks mean traceability; an amber route moves from an incoming alert through a redaction checkpoint to a sealed pine-green envelope. The result feels like a field instrument, not another observability dashboard.

## Palette

The light treatment is the primary one, chosen for long on-call reading sessions: `paper #F3F0E5`, `paper-raised #FCFAF2`, `ink #17241F`, `muted #526158`, `pine #194D3A`, `pine-deep #0D3529`, `amber #C76820`, `amber-dark #92420D`, `line #C8C8B8`, `danger #A1352C`, and `success #176A49`. All body copy and interactive states meet 4.5:1. The explicit dark treatment uses `night #101815`, `night-raised #18231F`, `chalk #F0EFE4`, `muted #AAB6AE`, `amber #F0A45B`, and `line #3F5149`. Dark-mode labels on the light `pine-deep` and `success #59C991` states use `night #101815` ink, giving 4.5:1 or better. It is activated by the user’s OS preference. Color is always accompanied by text or a symbol.

## Type and spacing

Headings use self-hosted **Fraunces**, whose slightly irregular serif forms evoke engraved map labels. Interface copy and numbers use self-hosted **Inter**, with tabular numerals for payload sizes, times, and fingerprints. At 700 px and below, the landing shell deliberately uses the installed system sans and Georgia-style serif instead: the 48 KB/67 KB webfont pair would otherwise compete with the mobile evidence-terrain LCP image. The palette, contour geometry, scale, and hierarchy preserve the field-instrument identity on phones while keeping repeatable mobile LCP under the product budget. The type scale is 14 / 16 / 20 / 28 / 44 / 64 px. The reading measure is 68 characters. Spacing follows a 4 px base with 8, 12, 16, 24, 32, 48, 64, and 96 px stops. Controls are at least 44 px high.

## Components and interaction grammar

- The route line is the principal organizing motif: alert → bound query → redact → sign → deliver.
- Surfaces use thin topographic rules, cropped coordinate labels, squared corners with one clipped corner, and low, directional shadows like layered survey paper.
- Primary controls are filled pine; safety-sensitive actions are outlined and name their exact consequence.
- A persistent status strip reports whether the relay is ready, waiting, or degraded. Success never relies on a transient toast alone.
- Configuration is a linear field checklist. Advanced upstream settings disclose in place. A generated hero illustration clarifies the transformation but contains no simulated UI or capability claims.
- On phones the illustration and secondary explanatory labels drop below the primary setup action; pipeline stages stack vertically; data tables become labelled blocks.
- On the 390 px demo, the completed envelope is the first work surface. Its service, error, first-seen time, bounds, and fingerprint use a compact survey readout before the editable alert.

## Motion

UI transitions last 180–240 ms and animate only opacity and transform. Pipeline stages reveal in route order and status changes use a single short “stamp” scale. Nothing loops. With `prefers-reduced-motion: reduce`, transforms and smooth scrolling are removed and state changes are immediate opacity swaps.

## Asset plan and provenance

One original raster hero depicts an abstract nocturnal topographic incident map resolving into a sealed evidence packet. It ships as responsive WebP below 300 KB on mobile, with explicit dimensions. All icons are original inline SVG using the same contour-line geometry.

**Prompt sheet:** “Editorial topographic cartography of a software incident: layered cream survey paper floating over deep pine terrain, precise contour lines and coordinate ticks, one burnt-amber alert route entering from the left, passing through a small redaction checkpoint, resolving into a compact sealed dark-green evidence envelope on the right, subtle ink texture, raking field-station light, oblique 35mm lens, restrained cream/pine/amber palette, sophisticated technical publication, abundant negative space, no humans, no screens, no dashboards, no text, no letters, no numbers, no logos, no watermark, no neon gradient, no glossy 3D, no corporate stock art.”

The image was generated specifically for this product with the factory Azure image deployment (`factory-image`) on 2026-08-27. Generated imagery is original and disclosed in the footer. Prompt metadata is retained beside source assets. The mobile WebP is a reviewed 720 × 480 derivative of that source (40,982 bytes); the desktop WebP remains 1536 × 1024. The 1200 × 630 social card and 180 px touch icon are crops of the same original source, produced locally on 2026-08-30.
