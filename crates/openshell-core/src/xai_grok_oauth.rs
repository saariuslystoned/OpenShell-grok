// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Experimental xAI Grok subscription OAuth compatibility boundary.
//!
//! xAI publishes OIDC discovery at `https://auth.x.ai/.well-known/openid-configuration`
//! and a device-code grant. It does not publish a stable third-party client
//! registration for OpenShell. This module is the single versioned home for
//! those compatibility constants. Do not copy them through the tree.
//!
//! The public `client_id` is the Grok CLI / shared xAI OAuth client used by
//! existing open-source agents. OpenShell creates its own grant and must not
//! import a live OpenClaw `xai` refresh token.

use url::Url;

/// Compatibility snapshot date for this experimental contract.
pub const COMPAT_REVISION: &str = "2026-08-13.1";

/// Canonical provider profile id.
pub const PROVIDER_TYPE: &str = "xai-grok-oauth";

/// User-facing alias accepted by `--type`.
pub const PROVIDER_TYPE_ALIAS: &str = "grok-subscription";

/// Injectable-looking access token key. The gateway uses it for
/// `inference.local` only; it is never written into sandbox env.
pub const ACCESS_TOKEN_KEY: &str = "XAI_GROK_ACCESS_TOKEN";

/// Optional model override stored on the provider config map.
pub const MODEL_CONFIG_KEY: &str = "XAI_GROK_MODEL";

/// Default model for the first inference-only slice.
pub const DEFAULT_MODEL: &str = "grok-4.6";

/// Pinned HTTPS inference origin. Base-URL overrides are rejected.
pub const INFERENCE_BASE_URL: &str = "https://api.x.ai/v1";

/// Published OIDC discovery document.
pub const DISCOVERY_URL: &str = "https://auth.x.ai/.well-known/openid-configuration";

/// Default token endpoint from the 2026-08-13 discovery snapshot.
pub const TOKEN_URL: &str = "https://auth.x.ai/oauth2/token";

/// Default device-code endpoint from the 2026-08-13 discovery snapshot.
pub const DEVICE_AUTHORIZATION_URL: &str = "https://auth.x.ai/oauth2/device/code";

/// Default revocation endpoint from the 2026-08-13 discovery snapshot.
pub const REVOCATION_URL: &str = "https://auth.x.ai/oauth2/revoke";

/// Shared public xAI / Grok CLI OAuth client id.
///
/// This is not an OpenShell-issued client. Treat the provider as experimental
/// until xAI publishes a first-party third-party integration contract.
pub const PUBLIC_CLIENT_ID: &str = "b1a00492-073a-47ea-816f-4c329264a828";

/// Device-code / refresh scopes. `offline_access` is required for a refresh token.
pub const SCOPES: &[&str] = &["grok-cli:access", "offline_access"];

const ALLOWED_INFERENCE_HOSTS: &[&str] = &["api.x.ai"];
const ALLOWED_AUTH_HOSTS: &[&str] = &["auth.x.ai", "accounts.x.ai"];

/// RFC 8628 device-code grant type.
pub const DEVICE_CODE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

#[must_use]
pub fn scope_param() -> String {
    SCOPES.join(" ")
}

#[must_use]
pub fn is_xai_grok_oauth_type(input: &str) -> bool {
    matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "xai-grok-oauth" | "grok-subscription" | "xai-oauth" | "grok-oauth" | "xai_grok_oauth"
    )
}

/// Personal subscription grants that must never become gateway-wide routes.
#[must_use]
pub fn is_personal_subscription_provider_type(input: &str) -> bool {
    is_xai_grok_oauth_type(input)
}

#[must_use]
pub fn is_sandbox_non_injectable_credential(provider_type: &str, key: &str) -> bool {
    if is_xai_grok_oauth_type(provider_type) {
        return matches!(
            key,
            ACCESS_TOKEN_KEY | "refresh_token" | "XAI_GROK_REFRESH_TOKEN" | "client_secret"
        );
    }
    false
}

#[must_use]
pub fn is_allowed_inference_base_url(raw: &str) -> bool {
    allowed_https_origin(raw, ALLOWED_INFERENCE_HOSTS)
}

#[must_use]
pub fn is_allowed_auth_url(raw: &str) -> bool {
    allowed_https_origin(raw, ALLOWED_AUTH_HOSTS)
}

fn allowed_https_origin(raw: &str, hosts: &[&str]) -> bool {
    let Ok(url) = Url::parse(raw.trim()) else {
        return false;
    };
    if url.scheme() != "https" {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    hosts
        .iter()
        .any(|allowed| host.eq_ignore_ascii_case(allowed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_resolve() {
        assert!(is_xai_grok_oauth_type("grok-subscription"));
        assert!(is_xai_grok_oauth_type("XAI-GROK-OAUTH"));
        assert!(is_personal_subscription_provider_type("xai-oauth"));
        assert!(!is_xai_grok_oauth_type("openai"));
        assert!(!is_personal_subscription_provider_type("codex"));
    }

    #[test]
    fn pins_inference_origin() {
        assert!(is_allowed_inference_base_url("https://api.x.ai/v1"));
        assert!(is_allowed_inference_base_url(
            "https://api.x.ai/v1/responses"
        ));
        assert!(!is_allowed_inference_base_url("https://evil.example/v1"));
        assert!(!is_allowed_inference_base_url("http://api.x.ai/v1"));
        assert!(!is_allowed_inference_base_url(
            "https://api.x.ai.evil.example/v1"
        ));
    }

    #[test]
    fn pins_auth_origin() {
        assert!(is_allowed_auth_url(TOKEN_URL));
        assert!(is_allowed_auth_url(DEVICE_AUTHORIZATION_URL));
        assert!(is_allowed_auth_url("https://accounts.x.ai/authorize"));
        assert!(!is_allowed_auth_url(
            "https://login.microsoftonline.com/token"
        ));
    }

    #[test]
    fn access_token_is_not_sandbox_injectable() {
        assert!(is_sandbox_non_injectable_credential(
            PROVIDER_TYPE,
            ACCESS_TOKEN_KEY
        ));
        assert!(!is_sandbox_non_injectable_credential(
            "openai",
            ACCESS_TOKEN_KEY
        ));
    }
}
