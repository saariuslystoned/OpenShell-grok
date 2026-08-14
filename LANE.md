# SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

# OpenShell-grok lane

This repository is a personal implementation lane for
[NVIDIA/OpenShell#2742](https://github.com/NVIDIA/OpenShell/issues/2742)
(gateway-owned xAI Grok subscription OAuth).

It is **not** an official NVIDIA repository and is **not** a GitHub fork.
`saariuslystoned/OpenShell` already occupies the one-fork slot and tracks
[NVIDIA/OpenShell#2740](https://github.com/NVIDIA/OpenShell/issues/2740)
(Codex subscription OAuth).

| Lane | Repository | Upstream issue |
| --- | --- | --- |
| Codex | [saariuslystoned/OpenShell](https://github.com/saariuslystoned/OpenShell) | #2740 |
| Grok | [saariuslystoned/OpenShell-grok](https://github.com/saariuslystoned/OpenShell-grok) | #2742 |

## Remotes

- `origin` — this lane (`saariuslystoned/OpenShell-grok`)
- `upstream` — `NVIDIA/OpenShell`

Sync with `git fetch upstream` and rebase onto `upstream/main` before proposing
an upstream PR. Do not open an upstream PR until the account is vouched and
#2742 is `state:accepted`.

## Scope

First slice is inference only: an OpenShell-owned xAI grant, gateway-only
refresh, sandbox-scoped `inference.local`, and pinned `api.x.ai` origins.
Do not import a live OpenClaw `xai` refresh token.
