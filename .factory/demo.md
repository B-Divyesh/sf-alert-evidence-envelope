# Demo sandbox

Open `https://alert-evidence-envelope.sociobot.in/demo`, or select **Try it with sample data** on the first screen.

The demo creates a random server workspace that expires after 24 hours. Its identifier and last rendered envelope use the browser keys `demo:alert-evidence-envelope:session` and `demo:alert-evidence-envelope:preview`.

SQLite stores only the random session ID and expiry time under `/data`. It never stores the sample alert or envelope. The session cannot read or write route configuration or delivery history.

The sample alert describes a checkout timeout. It includes two evidence rows, a private email, and a token. The page immediately builds an envelope that exposes the service, error, and first-seen time while replacing the email and token with `[REDACTED]`.

**Reset demo** deletes the current server session, clears the cached sample result, creates a new workspace, and restores the shipped alert. **Start for real** deletes the demo keys and returns to the protected route builder.

After one online visit, the service worker and demo namespace retain the shell and last sample result for an offline reload. No demo payload enters the production database.
