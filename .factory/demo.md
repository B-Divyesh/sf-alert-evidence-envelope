# Demo sandbox

Open `https://alert-evidence-envelope.sociobot.in/?demo=1`, or select **Try it with sample data** on the first screen. `/demo` remains a supported direct route.

The demo creates a random server workspace that expires after 24 hours. Its identifier and last rendered envelope use the browser keys `demo:alert-evidence-envelope:session` and `demo:alert-evidence-envelope:preview`.

SQLite stores only the random session ID and expiry time under `/data`. It never stores the sample alert or envelope. The session cannot read or write route configuration or delivery history.

The sample alert describes a checkout timeout. It includes two evidence rows, a private email, and a token. The page immediately builds an envelope that exposes the service, error, and first-seen time.

The **Sample alert JSON** editor controls the next sample preview. Invalid JSON is rejected in the browser before any preview request. **Restore valid sample** puts the shipped alert back so the preview can run again.

The demo ships two isolated sample routes. **Internal Slack** removes `token` before a Slack-style delivery. **Customer automation** removes both `email` and `token` before a JSON-webhook delivery. Switching routes only changes the `demo:alert-evidence-envelope:route` browser key and sends a fresh request to the demo session endpoint. It never reads or writes protected routes.

**Reset demo** deletes the current server session, clears the cached sample result, creates a new workspace, and restores the shipped alert. **Start for real** deletes the demo keys and returns to the protected route builder.

After one online visit, the service worker and demo namespace retain the shell and last sample result for an offline reload. No demo payload enters the production database.
