// The canonical list of commands the front end may invoke.
//
// `build.rs` `include!`s this file to generate an `allow-<command>` permission for each name
// (`AppManifest::commands`), which is what turns the capability files in `capabilities/` from
// documentation into enforcement: once the app has an ACL manifest, Tauri rejects any command
// a window's capability does not grant.
//
// Adding a command therefore means three edits — the handler, this list, and the capability of
// whichever window is allowed to call it. The tests below fail if any of the three is missed,
// rather than leaving it to be discovered as a rejected IPC call during an event.
//
// Plain `//` comments on purpose: `build.rs` includes this file inline, and an inner doc
// comment cannot appear in the middle of a file.

/// Every command registered in `generate_handler!`, in the same order.
pub const COMMANDS: &[&str] = &[
    "list_microphones",
    "list_outputs",
    "list_applications",
    "has_api_key",
    "set_api_key",
    "clear_api_key",
    "ondevice_readiness",
    "whisper_cpu_support",
    "whisper_models",
    "download_whisper_model",
    "cancel_whisper_download",
    "remove_whisper_model",
    "discard_whisper_pending",
    "start_session",
    "stop_session",
    "pause_session",
    "start_audio_test",
    "stop_audio_test",
    "set_overlay_click_through",
    "show_overlay",
    "get_overlay_placement",
    "set_overlay_placement",
    "display_layout",
    "recover_overlay_placement",
    "report_overlay_state",
    "start_overlay_drag",
    "nudge_overlay",
    "snap_overlay_to_bottom",
    "undo_overlay_move",
    "save_transcript",
    "text_scale_factor",
    "write_recovery",
    "read_recovery",
    "clear_recovery",
    "write_history",
    "append_history",
    "list_history",
    "delete_history",
    "rename_history",
    "ack_close",
    "set_close_guard",
    "set_close_to_tray",
    "hide_to_tray",
    "show_operator",
    "confirm_close",
    "set_tray_state",
    "open_support_page",
    "start_zoom_captions",
    "stop_zoom_captions",
    "send_zoom_test_caption",
    "zoom_captions_status",
    "open_zoom_guide",
];

#[cfg(test)]
mod tests {
    use super::COMMANDS;
    use serde_json::Value;

    const OPERATOR: &str = include_str!("../capabilities/operator.json");
    const OVERLAY: &str = include_str!("../capabilities/overlay.json");
    const LIB: &str = include_str!("lib.rs");

    /// The `allow-…` entries of a capability that name an app command — anything with a
    /// `prefix:` belongs to core or a plugin and is not this module's business.
    fn granted_commands(capability: &str) -> Vec<String> {
        let json: Value = serde_json::from_str(capability).expect("capability is not valid JSON");
        json["permissions"]
            .as_array()
            .expect("capability has no permissions array")
            .iter()
            .filter_map(|p| p.as_str())
            .filter(|p| !p.contains(':'))
            .map(|p| {
                p.strip_prefix("allow-")
                    .unwrap_or_else(|| panic!("{p} is neither prefixed nor an allow- permission"))
                    .replace('-', "_")
            })
            .collect()
    }

    fn windows(capability: &str) -> Vec<String> {
        let json: Value = serde_json::from_str(capability).unwrap();
        json["windows"]
            .as_array()
            .expect("capability has no windows array")
            .iter()
            .map(|w| w.as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn each_capability_covers_exactly_one_window() {
        assert_eq!(windows(OPERATOR), ["operator"]);
        assert_eq!(windows(OVERLAY), ["overlay"]);
    }

    /// Every entry of a capability, core and plugin permissions included.
    fn permissions(capability: &str) -> Vec<String> {
        let json: Value = serde_json::from_str(capability).unwrap();
        json["permissions"]
            .as_array()
            .expect("capability has no permissions array")
            .iter()
            .map(|p| {
                p.as_str()
                    .expect("a scoped permission object; the overlay has none")
                    .to_string()
            })
            .collect()
    }

    /// The point of the split (issue #31). The overlay is a caption surface: it repositions
    /// itself, turns its own click-through back on and tells the operator window what happened
    /// on it, and that is the whole list. Credentials, sessions, capture, the transcript and the
    /// close/quit path are the operator's alone.
    #[test]
    fn the_overlay_may_only_place_itself() {
        assert_eq!(
            granted_commands(OVERLAY),
            [
                "set_overlay_click_through",
                "get_overlay_placement",
                "report_overlay_state",
                "start_overlay_drag",
                "nudge_overlay",
                "snap_overlay_to_bottom",
                "undo_overlay_move",
            ]
        );
    }

    /// Core permissions too, pinned whole (B4). `core:event:default` would let the overlay emit
    /// any event under any name — a `tray-command` that stops a billable session, captions that
    /// reach the transcript — and the core window setters take any window's label, so they
    /// would let it move the operator window. It listens, and the commands above do the rest.
    /// A new entry here is a decision about what an audience-facing window may do, so it has
    /// to be made in this test as well as in the file.
    #[test]
    fn the_overlay_capability_is_pinned_whole() {
        assert_eq!(
            permissions(OVERLAY),
            [
                "core:event:allow-listen",
                "core:event:allow-unlisten",
                "allow-set-overlay-click-through",
                "allow-get-overlay-placement",
                "allow-report-overlay-state",
                "allow-start-overlay-drag",
                "allow-nudge-overlay",
                "allow-snap-overlay-to-bottom",
                "allow-undo-overlay-move",
            ]
        );
    }

    /// The overlay talks to the operator window through `report_overlay_state` alone, so the
    /// operator never needs to be granted it, nor the overlay's own placement commands.
    #[test]
    fn the_operator_is_not_granted_the_overlay_only_commands() {
        let operator = granted_commands(OPERATOR);
        for command in [
            "report_overlay_state",
            "start_overlay_drag",
            "nudge_overlay",
            "snap_overlay_to_bottom",
            "undo_overlay_move",
        ] {
            assert!(
                !operator.iter().any(|g| g == command),
                "`{command}` is the overlay's own"
            );
        }
    }

    #[test]
    fn every_command_is_granted_to_some_window() {
        let granted: Vec<String> = granted_commands(OPERATOR)
            .into_iter()
            .chain(granted_commands(OVERLAY))
            .collect();

        for command in COMMANDS {
            assert!(
                granted.iter().any(|g| g == command),
                "`{command}` is registered but no capability grants it, so every call to it \
                 would be rejected by the ACL at runtime"
            );
        }
    }

    #[test]
    fn no_capability_grants_a_command_that_does_not_exist() {
        for granted in granted_commands(OPERATOR)
            .into_iter()
            .chain(granted_commands(OVERLAY))
        {
            assert!(
                COMMANDS.contains(&granted.as_str()),
                "a capability grants `{granted}`, which is not a registered command"
            );
        }
    }

    /// `generate_handler!` is a macro, so nothing at runtime can compare its list with this
    /// one. Reading the source is crude but it is the only thing that catches a command added
    /// to the handler and forgotten here — which would build cleanly and fail on first use.
    #[test]
    fn the_list_matches_the_handler() {
        let handler = LIB
            .split_once("generate_handler![")
            .expect("lib.rs no longer registers commands with generate_handler!")
            .1
            .split_once(']')
            .unwrap()
            .0;

        let registered: Vec<&str> = handler
            .lines()
            .filter_map(|line| line.trim().strip_suffix(','))
            .filter_map(|entry| entry.rsplit("::").next())
            .collect();

        assert_eq!(
            registered, COMMANDS,
            "the handler and COMMANDS have drifted apart"
        );
    }
}
