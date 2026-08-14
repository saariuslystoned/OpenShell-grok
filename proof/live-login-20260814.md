# SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

# Live login proof — 2026-08-14

Throwaway gateway on spark-2. Production NemoClaw gateway was not replaced.
Proof state (TLS + sqlite grant) was deleted after capture.

## Setup

- Host: spark-2 (`192.168.1.41`)
- Source: `~/Developer/OpenShell-grok` (lane for NVIDIA/OpenShell#2742)
- Binaries: `target/release/openshell` and `openshell-gateway` (`0.0.0` local build)
- Proof gateway: `https://127.0.0.1:18080` name `grok-proof`
- Production: `nemoclaw` `https://127.0.0.1:8080` `0.0.85` left running

## Login

```text
openshell --gateway grok-proof provider create \
  --name grok-sub \
  --type grok-subscription \
  --login-xai-oauth \
  --config XAI_GROK_MODEL=grok-4.6
```

Device code `PDC7-CHNA` approved in browser.
CLI reported: created provider `grok-sub`; OpenClaw xAI login was not used.
Login log contained no JWT-like material.

## Provider record (no secret values)

```text
NAME      TYPE            CREDENTIAL_KEYS   CONFIG_KEYS
grok-sub  xai-grok-oauth  1                 1

Provider:
  Name: grok-sub
  Type: xai-grok-oauth
  Credential keys: XAI_GROK_ACCESS_TOKEN
  Config keys: XAI_GROK_MODEL
```

`provider get` did not print access/refresh token values.

## Refresh

Immediately after login:

```text
STATUS=configured  STRATEGY=oauth2_refresh_token
EXPIRES_AT=2026-08-14 07:57:24
```

After `provider refresh rotate grok-sub --credential-key XAI_GROK_ACCESS_TOKEN`:

```text
Rotation requested ... (refreshed)
STATUS=refreshed
LAST_REFRESH=2026-08-14 01:57:54
EXPIRES_AT=2026-08-14 02:57:54
LAST_ERROR=-
```

Rotate used the OpenShell-owned grant against pinned `auth.x.ai`. No token material in subsequent `provider get`.

## Production after teardown

```text
* nemoclaw  https://127.0.0.1:8080  local  user  mtls  Connected  0.0.85
sandbox spark02-assistant  Ready
port 18080 gone
~/Developer/openshell-grok-proof removed
```

## Later production OpenClaw proof

See `proof/openclaw-grok-20260814.md`. Screenshots are in
[saari-co/public-oss-proof-assets](https://github.com/saari-co/public-oss-proof-assets/tree/main/openshell-grok/2742/2026-08-14),
not this tree.

## Not in this proof

- Dual-proof isolation vs Codex #2740
- Upstream NVIDIA/OpenShell PR
