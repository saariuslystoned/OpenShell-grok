# OpenClaw grok-sub proof — 2026-08-14

Production NemoClaw gateway on spark-2. `grok-sub` is attached to `spark02-assistant`.
No OpenClaw xAI tokens were imported. No token values are recorded here.

## What was wrong at first

OpenClaw's saved default was `inference/nemotron-3.5-lightning`. Sending on Main Session
without swapping models made the assistant name itself Nemotron even after `grok-sub`
was attached. `qwen3.6` is retired leftover chrome; it is not the live default.

The first `xai-grok-oauth` grant only had `grok-cli:access`. `https://api.x.ai` rejected
chat with HTTP 403 `OAuth2 token missing required scope: api:access`. A new device-code
grant (`7CEK-WNBS`) added `api:access`.

## Sandbox `inference.local` (xAI, not Nemotron)

From `spark02-assistant` after the `api:access` grant:

```text
GET  https://inference.local/v1/models                 HTTP 200  (xAI catalog includes grok-4.6)
POST https://inference.local/v1/chat/completions
     model=grok-4.6
     -> HTTP 200  content=grok-subscription-proof-ok
```

Supervisor log for that probe:

```text
ALLOWED inference.local:443
routing proxy inference request endpoint=https://api.x.ai/v1 path=/v1/chat/completions
```

Gateway-wide `openshell inference get` is still `compatible-endpoint` /
`nemotron-3.5-lightning`. Personal Grok grants are sandbox-scoped: only a sandbox
with `grok-sub` attached receives the xAI route.

## OpenClaw: swap, save, new conversation

1. Saved `agents.defaults.model.primary` to `grok-sub/grok-4.6` in
   `/sandbox/.openclaw/openclaw.json`. Nemotron stays listed as the local model.
2. Opened the NemoClaw dashboard in Chrome on the MacBook via the existing
   `127.0.0.1:28789` tunnel.
3. Clicked **New session** (did not reuse Main Session).
4. Sent: `Reply with exactly this line and nothing else: GROKSUB-PROOF-20260814`

Visible UI after the turn:

```text
You:        Reply with exactly this line and nothing else: GROKSUB-PROOF-20260814
Assistant:  GROKSUB-PROOF-20260814
Footer:     inference/grok-4.6 · Off
```

Screenshot (hosted in [saari-co/public-oss-proof-assets](https://github.com/saari-co/public-oss-proof-assets)):

![OpenClaw grok-4.6 proof turn](https://raw.githubusercontent.com/saari-co/public-oss-proof-assets/main/openshell-grok/2742/2026-08-14/openclaw-conversation-grok-20260814.png)

- Conversation: https://github.com/saari-co/public-oss-proof-assets/blob/main/openshell-grok/2742/2026-08-14/openclaw-conversation-grok-20260814.png
- Earlier empty Main Session (leftover `qwen3.6` chrome): https://github.com/saari-co/public-oss-proof-assets/blob/main/openshell-grok/2742/2026-08-14/openclaw-conversation-20260814.png
- Before swap (`nemotron-3.5-lightning`): https://github.com/saari-co/public-oss-proof-assets/blob/main/openshell-grok/2742/2026-08-14/openclaw-01-before-swap.png

The footer label is the OpenClaw display name `inference/grok-4.6`. The provider
record is `grok-sub` (`xai-grok-oauth`); the sandbox supervisor still pins that
route to `https://api.x.ai/v1`.

## Not in this packet

- Restore of the OpenClaw default back to Nemotron (left on `grok-sub/grok-4.6`
  after the proof turn)
- Dual-proof isolation vs Codex #2740
- Upstream NVIDIA/OpenShell PR (still waiting on discussion #2741 vouch)
