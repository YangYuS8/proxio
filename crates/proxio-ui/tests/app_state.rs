use proxio_diagnose::{CheckReport, EffectiveProxy, LayerReport, LayerStatus, TransportMode};
use proxio_ui::app::{ActionSummary, AppState};
use proxio_ui::services::{ActionResult, LoadedState};

#[test]
fn loads_profile_names_and_current_selection_into_state() {
    let state = AppState::from_loaded(
        vec!["direct".into(), "proxy".into()],
        Some("proxy".into()),
        "proxied".into(),
    );

    assert_eq!(state.profile_names, vec!["direct", "proxy"]);
    assert_eq!(state.current_profile.as_deref(), Some("proxy"));
    assert_eq!(state.selected_profile.as_deref(), Some("proxy"));
}

#[test]
fn records_action_summary_and_check_report() {
    let mut state = AppState::default();
    state.action_summary = Some(ActionSummary {
        success_count: 2,
        skipped_count: 1,
        failed_count: 0,
        message: "applied profile".into(),
    });
    state.last_check = Some(CheckReport {
        target_url: "https://example.com".into(),
        profile_name: "proxy".into(),
        transport: EffectiveProxy {
            mode: TransportMode::Proxied,
            value: Some("http://127.0.0.1:7890".into()),
        },
        dns: LayerReport {
            status: LayerStatus::Success,
            summary: "resolved".into(),
            detail: String::new(),
        },
        tcp: LayerReport {
            status: LayerStatus::Success,
            summary: "connected".into(),
            detail: String::new(),
        },
        tls: LayerReport {
            status: LayerStatus::Success,
            summary: "tls ok".into(),
            detail: String::new(),
        },
        http: LayerReport {
            status: LayerStatus::Success,
            summary: "http ok".into(),
            detail: String::new(),
        },
        conclusion: "ok".into(),
    });

    assert_eq!(state.action_summary.as_ref().unwrap().success_count, 2);
    assert_eq!(state.last_check.as_ref().unwrap().profile_name, "proxy");
}

#[test]
fn applies_transition_helpers_for_selection_results_and_errors() {
    let mut state = AppState::default();

    state.set_loaded(LoadedState {
        profile_names: vec!["direct".into(), "proxy".into()],
        current_profile: Some("proxy".into()),
        mode_summary: "proxied".into(),
    });
    state.set_selected_profile("direct".into());
    state.set_action_result(ActionResult {
        success_count: 1,
        skipped_count: 0,
        failed_count: 0,
        message: "applied profile".into(),
    });
    state.set_check_result(sample_check_report());
    state.set_error("apply failed".into());

    assert_eq!(state.selected_profile.as_deref(), Some("direct"));
    assert_eq!(state.current_profile.as_deref(), Some("proxy"));
    assert_eq!(
        state.action_summary.as_ref().unwrap().message,
        "applied profile"
    );
    assert_eq!(
        state.last_check.as_ref().unwrap().target_url,
        "https://example.com"
    );
    assert_eq!(state.error_message.as_deref(), Some("apply failed"));
}

fn sample_check_report() -> CheckReport {
    CheckReport {
        target_url: "https://example.com".into(),
        profile_name: "proxy".into(),
        transport: EffectiveProxy {
            mode: TransportMode::Proxied,
            value: Some("http://127.0.0.1:7890".into()),
        },
        dns: LayerReport {
            status: LayerStatus::Success,
            summary: "resolved".into(),
            detail: String::new(),
        },
        tcp: LayerReport {
            status: LayerStatus::Success,
            summary: "connected".into(),
            detail: String::new(),
        },
        tls: LayerReport {
            status: LayerStatus::Success,
            summary: "tls ok".into(),
            detail: String::new(),
        },
        http: LayerReport {
            status: LayerStatus::Success,
            summary: "http ok".into(),
            detail: String::new(),
        },
        conclusion: "ok".into(),
    }
}
