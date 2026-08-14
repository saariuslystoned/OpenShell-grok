// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Attended xAI device-code login for the experimental Grok subscription provider.
//!
//! Creates an OpenShell-owned grant. Never reads OpenClaw auth profiles.

use std::time::Duration;

use miette::{IntoDiagnostic, Result};
use openshell_core::xai_grok_oauth::{
    DEVICE_AUTHORIZATION_URL, DEVICE_CODE_GRANT_TYPE, PUBLIC_CLIENT_ID, TOKEN_URL,
    is_allowed_auth_url, scope_param,
};
use owo_colors::OwoColorize;
use serde::Deserialize;

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_POLL_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Debug, Clone)]
pub struct XaiGrokOauthGrant {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: Option<String>,
    verification_uri_complete: Option<String>,
    expires_in: Option<u64>,
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TokenSuccess {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TokenErrorBody {
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeviceCodePoll {
    Pending { slow_down: bool },
    Terminal(String),
}

#[must_use]
pub fn classify_device_code_error(status: u16, body: &str) -> DeviceCodePoll {
    let parsed = serde_json::from_str::<TokenErrorBody>(body).ok();
    let code = parsed
        .as_ref()
        .and_then(|value| value.error.as_deref())
        .unwrap_or("");
    match (status, code) {
        (_, "authorization_pending") => DeviceCodePoll::Pending { slow_down: false },
        (_, "slow_down") => DeviceCodePoll::Pending { slow_down: true },
        (408 | 429, _) => DeviceCodePoll::Pending { slow_down: true },
        (500..=599, _) => DeviceCodePoll::Pending { slow_down: false },
        (_, "expired_token" | "access_denied" | "invalid_grant") => DeviceCodePoll::Terminal(
            parsed
                .and_then(|value| value.error_description)
                .filter(|desc| !desc.trim().is_empty())
                .unwrap_or_else(|| {
                    "xAI Grok OAuth grant was rejected or expired; re-run --login-xai-oauth"
                        .to_string()
                }),
        ),
        _ => DeviceCodePoll::Terminal(format!(
            "xAI token endpoint returned HTTP {status}; re-run --login-xai-oauth"
        )),
    }
}

pub async fn run_device_code_login() -> Result<XaiGrokOauthGrant> {
    if !is_allowed_auth_url(DEVICE_AUTHORIZATION_URL) || !is_allowed_auth_url(TOKEN_URL) {
        return Err(miette::miette!(
            "xAI OAuth compatibility endpoints failed the pinned-origin check"
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .into_diagnostic()?;

    let device = request_device_code(&client).await?;
    let verification = device
        .verification_uri_complete
        .as_deref()
        .or(device.verification_uri.as_deref())
        .unwrap_or("https://accounts.x.ai");

    println!(
        "{}",
        "Sign in to xAI for an OpenShell-owned Grok grant (not an OpenClaw import)."
            .cyan()
            .bold()
    );
    println!();
    println!("  Open: {}", verification.bold());
    println!("  Code: {}", device.user_code.bold());
    println!();
    println!("Waiting for approval...");

    let deadline = std::time::Instant::now()
        + Duration::from_secs(device.expires_in.filter(|value| *value > 0).unwrap_or(900));
    let mut interval = Duration::from_secs(device.interval.unwrap_or(5)).max(DEFAULT_POLL_INTERVAL);

    loop {
        if std::time::Instant::now() >= deadline {
            return Err(miette::miette!(
                "xAI device-code approval timed out; re-run --login-xai-oauth"
            ));
        }
        tokio::time::sleep(interval).await;
        match poll_token(&client, &device.device_code).await? {
            PollResult::Granted(grant) => return Ok(grant),
            PollResult::Pending { slow_down } => {
                if slow_down {
                    interval = (interval + Duration::from_secs(5)).min(MAX_POLL_INTERVAL);
                }
            }
        }
    }
}

async fn request_device_code(client: &reqwest::Client) -> Result<DeviceCodeResponse> {
    let response = client
        .post(DEVICE_AUTHORIZATION_URL)
        .form(&[("client_id", PUBLIC_CLIENT_ID), ("scope", &scope_param())])
        .send()
        .await
        .into_diagnostic()?;
    let status = response.status();
    let body = response.text().await.into_diagnostic()?;
    if !status.is_success() {
        return Err(miette::miette!(
            "xAI device-code request failed with HTTP {status}"
        ));
    }
    serde_json::from_str(&body).into_diagnostic()
}

enum PollResult {
    Granted(XaiGrokOauthGrant),
    Pending { slow_down: bool },
}

async fn poll_token(client: &reqwest::Client, device_code: &str) -> Result<PollResult> {
    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", DEVICE_CODE_GRANT_TYPE),
            ("device_code", device_code),
            ("client_id", PUBLIC_CLIENT_ID),
        ])
        .send()
        .await
        .into_diagnostic()?;
    let status = response.status();
    let body = response.text().await.into_diagnostic()?;
    if status.is_success() {
        let token: TokenSuccess = serde_json::from_str(&body).into_diagnostic()?;
        let refresh_token = token
            .refresh_token
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                miette::miette!(
                    "xAI token response omitted refresh_token; cannot store a gateway-owned grant"
                )
            })?;
        if token.access_token.trim().is_empty() {
            return Err(miette::miette!("xAI token response omitted access_token"));
        }
        return Ok(PollResult::Granted(XaiGrokOauthGrant {
            access_token: token.access_token,
            refresh_token,
            expires_in: token.expires_in,
        }));
    }
    match classify_device_code_error(status.as_u16(), &body) {
        DeviceCodePoll::Pending { slow_down } => Ok(PollResult::Pending { slow_down }),
        DeviceCodePoll::Terminal(message) => Err(miette::miette!("{message}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_and_retryable_errors_keep_polling() {
        assert_eq!(
            classify_device_code_error(400, r#"{"error":"authorization_pending"}"#),
            DeviceCodePoll::Pending { slow_down: false }
        );
        assert_eq!(
            classify_device_code_error(400, r#"{"error":"slow_down"}"#),
            DeviceCodePoll::Pending { slow_down: true }
        );
        assert_eq!(
            classify_device_code_error(429, "{}"),
            DeviceCodePoll::Pending { slow_down: true }
        );
        assert_eq!(
            classify_device_code_error(503, "{}"),
            DeviceCodePoll::Pending { slow_down: false }
        );
    }

    #[test]
    fn terminal_grant_errors_fail_closed() {
        match classify_device_code_error(400, r#"{"error":"invalid_grant"}"#) {
            DeviceCodePoll::Terminal(message) => {
                assert!(message.contains("re-run --login-xai-oauth"));
            }
            other => panic!("expected terminal poll, got {other:?}"),
        }
        match classify_device_code_error(403, r#"{"error":"access_denied"}"#) {
            DeviceCodePoll::Terminal(_) => {}
            other => panic!("expected terminal poll, got {other:?}"),
        }
    }
}
