// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::*;

use crate::policy::{self, PermissionRequest, PermissionRequestKind};

wrap_permission_handler! {
  pub struct TauriCefPermissionHandler {
    webview_label: String,
  }

  impl PermissionHandler {
    fn on_request_media_access_permission(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut MediaAccessCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(callback) = callback else {
        return 0;
      };
      let origin = requesting_origin.map(|o| o.to_string()).unwrap_or_default();
      let request = PermissionRequest {
        webview_label: &self.webview_label,
        requesting_origin: &origin,
        permissions: requested_permissions,
        kind: PermissionRequestKind::MediaAccess,
      };
      if policy::permission_allowed(&request) {
        callback.cont(requested_permissions);
      } else {
        log::info!(
          "denied media access ({requested_permissions:#x}) for origin {origin:?} in webview {:?}",
          self.webview_label
        );
        callback.cont(
          cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_NONE as u32,
        );
      }
      1
    }

    fn on_show_permission_prompt(
      &self,
      _browser: Option<&mut Browser>,
      _prompt_id: u64,
      requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut PermissionPromptCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(callback) = callback else {
        return 0;
      };
      let origin = requesting_origin.map(|o| o.to_string()).unwrap_or_default();
      let request = PermissionRequest {
        webview_label: &self.webview_label,
        requesting_origin: &origin,
        permissions: requested_permissions,
        kind: PermissionRequestKind::Prompt,
      };
      let result = if policy::permission_allowed(&request) {
        cef::sys::cef_permission_request_result_t::CEF_PERMISSION_RESULT_ACCEPT
      } else {
        log::info!(
          "denied permission prompt ({requested_permissions:#x}) for origin {origin:?} in webview {:?}",
          self.webview_label
        );
        cef::sys::cef_permission_request_result_t::CEF_PERMISSION_RESULT_DENY
      };
      callback.cont(PermissionRequestResult::from(result));
      1
    }
  }
}
