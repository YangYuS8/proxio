use proxio_diagnose::{CheckReport, EffectiveProxy, LayerReport, LayerStatus, TransportMode};
use proxio_ui::app::{AppState, DiagnosisRow};
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
fn updates_summary_and_error_banner_state() {
    let mut state = AppState::default();

    state.set_header_summary(Some("proxy".into()), "proxied".into());
    state.set_error("load failed".into());

    assert_eq!(state.current_profile.as_deref(), Some("proxy"));
    assert_eq!(state.mode_summary, "proxied");
    assert_eq!(state.error_message.as_deref(), Some("load failed"));
}

#[test]
fn records_action_summary_cards_and_diagnosis_rows() {
    let mut state = AppState::default();

    state.set_action_result(ActionResult {
        success_count: 2,
        skipped_count: 1,
        failed_count: 0,
        message: "applied profile".into(),
    });
    state.set_check_result(CheckReport {
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

    let action_summary = state.action_summary.as_ref().unwrap();
    assert_eq!(action_summary.success_count, 2);
    assert_eq!(action_summary.message, "applied profile");
    assert_eq!(state.diagnosis_rows.len(), 4);
    assert_eq!(state.diagnosis_rows[0].layer, "DNS");
    assert_eq!(state.diagnosis_rows[3].summary, "http ok");
    assert_eq!(state.last_check.as_ref().unwrap().profile_name, "proxy");
}

#[test]
fn keeps_action_and_diagnosis_sections_populated_together() {
    let mut state = AppState::default();

    state.set_action_result(ActionResult {
        success_count: 1,
        skipped_count: 0,
        failed_count: 0,
        message: "disabled managed settings".into(),
    });
    state.set_diagnosis_rows(vec![DiagnosisRow {
        layer: "DNS".into(),
        status: "Success".into(),
        summary: "resolved".into(),
        detail: None,
    }]);

    assert_eq!(
        state.action_summary.as_ref().unwrap().message,
        "disabled managed settings"
    );
    assert_eq!(state.diagnosis_rows.len(), 1);
    assert_eq!(state.diagnosis_rows[0].summary, "resolved");
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

#[test]
fn preserves_selection_use_apply_disable_and_check_state_transitions() {
    let mut state = AppState::from_loaded(
        vec!["direct".into(), "proxy".into()],
        Some("proxy".into()),
        "proxied".into(),
    );

    state.set_selected_profile("direct".into());
    assert_eq!(state.selected_profile.as_deref(), Some("direct"));
    assert_eq!(state.current_profile.as_deref(), Some("proxy"));

    state.set_header_summary(state.selected_profile.clone(), "direct".into());
    assert_eq!(state.current_profile.as_deref(), Some("direct"));
    assert_eq!(state.mode_summary, "direct");

    state.set_action_result(ActionResult {
        success_count: 1,
        skipped_count: 0,
        failed_count: 0,
        message: "applied profile".into(),
    });
    assert_eq!(
        state.action_summary.as_ref().unwrap().message,
        "applied profile"
    );

    state.set_action_result(ActionResult {
        success_count: 2,
        skipped_count: 1,
        failed_count: 0,
        message: "disabled managed settings".into(),
    });
    assert_eq!(
        state.action_summary.as_ref().unwrap().message,
        "disabled managed settings"
    );

    state.set_check_result(sample_check_report());
    assert_eq!(state.last_check.as_ref().unwrap().conclusion, "ok");
    assert_eq!(state.diagnosis_rows.len(), 4);
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
