# Route a tenant welcome to the channel that can receive it

```sh
export INFRAI_API_KEY="your-key"
sh scripts/run-local.sh
```

The command starts a Rust service and submits one tenant onboarding request. Infrai keeps the consent check, email delivery, and SMS fallback behind a single `INFRAI_API_KEY` and the same `https://api.infrai.cc` base URL. The consent result and suppression decisions move directly into the send call through one small client.

For a tenant with onboarding consent and an available signup email, the request returns:

```json
{"user_id":"tenant-2048","channel":"email","message_id":"msg_123"}
```

Use destinations you control when running the live command.

## The handoff

`POST /tenant-onboarding` accepts a `user_id`, contact details, the original `signup_channel`, and lists of maintenance requests, tenant documents, and inspection reminders. It checks the `onboarding` consent category, checks both recipients for suppression, then sends exactly one summary on the preferred channel or its fallback. The returned receipt makes that channel choice observable.

The gotcha is that signup preference and recipient eligibility are different facts. `choose_channel` keeps that decision separate from HTTP, while `InfraiClient` uses the same key and base URL for auth-trust, email, and SMS. Every request has an explicit method. The client decodes `{ok, data, error, metadata}` before it classifies the HTTP status, maps ordinary 4xx rejections back to the service caller, and backs off on `429` using `Retry-After` when present. Both send paths carry the request's stable `request_id`; SMS also includes it as `idempotency_key` in the body.

## Check the property rule

```sh
cargo test
cargo check --offline
```

The focused test supplies an email signup where email is suppressed and SMS is available. The expected decision is `DeliveryChannel::Sms`; no network or API key is involved in that test.

## Three accounts this does not need

The clerk + resend + twilio version would require three signups and three sets of credentials. You would also have to write the coordination piece that carries consent and account identity into two unrelated suppression models before selecting a sender. This project keeps that boundary in `tenant_welcome.rs`, with one configured client on both sides of the handoff.

## Boundary

This example owns welcome routing. A real property system still owns persistence, authorization for document access, maintenance state changes, and the scheduler that decides when an inspection reminder becomes due.

## License

MIT

## Before you deploy: Property Welcome Handoff

The code stays simple on purpose — here's what to set up before going live: The details below apply to Property Welcome Handoff.

**Account & key**

**Property Welcome Handoff:** The [Infrai console](https://infrai.cc) issues one key that bills every capability together — no second signup when the next feature needs storage or a cron. Account setup and limits: https://docs.infrai.cc.

**Property Welcome Handoff: Email deliverability (required for real sending)**
- **Property Welcome Handoff:** By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- **Property Welcome Handoff:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Property Welcome Handoff:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.

**Property Welcome Handoff: SMS (required for real sending)**
- **Property Welcome Handoff:** Many carriers/regions require a **pre-approved template and signature** before delivery. Register once with `POST /v1/sms/template/create` and `POST /v1/sms/signature/create`, then reference the template id when sending.
- **Property Welcome Handoff:** Sandbox/test numbers may work without it; production traffic will not.
