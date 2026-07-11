// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! App-settable policies for Chromium permission requests and popups.
//!
//! Chromium permission requests reaching CEF — media access
//! (`getUserMedia`/`getDisplayMedia`) and permission prompts (notifications,
//! geolocation, clipboard, …) — are answered by a process-global,
//! deny-biased policy, so remote web content an app embeds never inherits
//! the app UI's privileges.
//!
//! Defaults when no policy is set (or the policy returns
//! [`PermissionDecision::Default`]):
//! - app-local origins (the app's own custom schemes from
//!   [`crate::CefConfig::custom_schemes`], plus `http(s)` on `localhost`,
//!   loopback IPs and `*.localhost` dev hosts) get media capture and
//!   permission prompts — the app UI keeps working out of the box;
//! - every other origin is denied.

use std::sync::OnceLock;

/// What a [`PermissionRequest`] is asking for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionRequestKind {
  /// `getUserMedia` / `getDisplayMedia` — `permissions` is a mask of CEF's
  /// `cef_media_access_permission_types_t` bits.
  MediaAccess,
  /// A Chromium permission prompt (notifications, geolocation, clipboard, …)
  /// — `permissions` is a mask of `cef_permission_request_types_t` bits.
  Prompt,
}

/// A permission request routed through the policy set by
/// [`set_permission_policy`].
#[derive(Debug)]
pub struct PermissionRequest<'a> {
  /// Label of the tauri webview the request originated from.
  pub webview_label: &'a str,
  /// The origin requesting the permission (e.g. `https://example.com`).
  /// May be empty when CEF does not report one.
  pub requesting_origin: &'a str,
  /// Raw CEF permission mask; interpret according to `kind`.
  pub permissions: u32,
  pub kind: PermissionRequestKind,
}

/// A policy verdict for a [`PermissionRequest`] or popup request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
  Allow,
  Deny,
  /// Fall through to the crate default (app-local origins only).
  Default,
}

type PermissionPolicy = dyn Fn(&PermissionRequest<'_>) -> PermissionDecision + Send + Sync;

static PERMISSION_POLICY: OnceLock<Box<PermissionPolicy>> = OnceLock::new();

/// Sets the process-global permission policy. Call before
/// `tauri::Builder::run`. Later calls are ignored.
pub fn set_permission_policy(
  policy: impl Fn(&PermissionRequest<'_>) -> PermissionDecision + Send + Sync + 'static,
) {
  let _ = PERMISSION_POLICY.set(Box::new(policy));
}

pub(crate) fn permission_allowed(request: &PermissionRequest<'_>) -> bool {
  let decision = PERMISSION_POLICY
    .get()
    .map(|policy| policy(request))
    .unwrap_or(PermissionDecision::Default);
  match decision {
    PermissionDecision::Allow => true,
    PermissionDecision::Deny => false,
    PermissionDecision::Default => is_app_local_origin(request.requesting_origin),
  }
}

/// Whether `origin` belongs to the app itself rather than remote web content:
/// one of the app's registered custom schemes, or http(s) on a
/// localhost/loopback/`*.localhost` host (dev servers, `.localhost` protocol
/// domains).
pub fn is_app_local_origin(origin: &str) -> bool {
  let Ok(url) = url::Url::parse(origin) else {
    return false;
  };
  let scheme = url.scheme();
  if crate::config::config().custom_schemes.iter().any(|s| s == scheme) {
    return true;
  }
  if scheme == "http" || scheme == "https" {
    return matches!(
      url.host_str(),
      Some(host) if host == "localhost"
        || host == "127.0.0.1"
        || host == "[::1]"
        || host == "::1"
        || host.ends_with(".localhost")
    );
  }
  false
}

/// A `window.open` / popup request routed through the policy set by
/// [`set_popup_policy`].
#[derive(Debug)]
pub struct PopupRequest<'a> {
  /// Label of the tauri webview that initiated the popup.
  pub webview_label: &'a str,
  /// The URL the popup wants to open. Empty if CEF reports none.
  pub url: &'a str,
}

type PopupPolicy = dyn Fn(&PopupRequest<'_>) -> bool + Send + Sync;

static POPUP_POLICY: OnceLock<Box<PopupPolicy>> = OnceLock::new();

/// Sets the process-global popup policy (`true` = allow the popup as a
/// native CEF window). When unset, webviews whose tauri builder installed an
/// `on_new_window` handler deny popups and all others allow them — see
/// `life_span.rs` for why the tauri handler itself cannot be consulted.
pub fn set_popup_policy(policy: impl Fn(&PopupRequest<'_>) -> bool + Send + Sync + 'static) {
  let _ = POPUP_POLICY.set(Box::new(policy));
}

pub(crate) fn popup_allowed(request: &PopupRequest<'_>) -> Option<bool> {
  POPUP_POLICY.get().map(|policy| policy(request))
}
