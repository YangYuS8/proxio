use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Task};
use proxio_diagnose::{CheckReport, LayerReport};

use crate::services::{ActionResult, LoadedState, RealServices, UiServices};

#[derive(Debug, Clone, Default)]
pub struct ActionSummary {
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub profile_names: Vec<String>,
    pub current_profile: Option<String>,
    pub selected_profile: Option<String>,
    pub mode_summary: String,
    pub action_summary: Option<ActionSummary>,
    pub check_input: String,
    pub last_check: Option<CheckReport>,
    pub is_busy: bool,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn from_loaded(
        profile_names: Vec<String>,
        current_profile: Option<String>,
        mode_summary: String,
    ) -> Self {
        Self {
            selected_profile: current_profile.clone(),
            profile_names,
            current_profile,
            mode_summary,
            ..Self::default()
        }
    }

    pub fn set_selected_profile(&mut self, name: String) {
        self.selected_profile = Some(name);
        self.error_message = None;
    }

    pub fn set_loaded(&mut self, loaded: LoadedState) {
        self.profile_names = loaded.profile_names;
        self.current_profile = loaded.current_profile.clone();
        self.selected_profile = loaded.current_profile;
        self.mode_summary = loaded.mode_summary;
        self.is_busy = false;
        self.error_message = None;
    }

    pub fn set_action_result(&mut self, result: ActionResult) {
        self.action_summary = Some(ActionSummary {
            success_count: result.success_count,
            skipped_count: result.skipped_count,
            failed_count: result.failed_count,
            message: result.message,
        });
        self.is_busy = false;
        self.error_message = None;
    }

    pub fn set_check_result(&mut self, report: CheckReport) {
        self.last_check = Some(report);
        self.is_busy = false;
        self.error_message = None;
    }

    pub fn set_error(&mut self, message: String) {
        self.is_busy = false;
        self.error_message = Some(message);
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<LoadedState, String>),
    ProfileSelected(String),
    UseSelectedProfile,
    UsedProfile(Result<LoadedState, String>),
    ApplyPressed,
    Applied(Result<ActionResult, String>),
    DisablePressed,
    Disabled(Result<ActionResult, String>),
    CheckInputChanged(String),
    CheckPressed,
    Checked(Result<CheckReport, String>),
}

struct Shell {
    state: AppState,
    services: RealServices,
}

pub fn run() -> iced::Result {
    iced::application("Proxio", update, view).run_with(|| {
        let services = RealServices::new().expect("services init");
        let load_services = services.clone();

        (
            Shell {
                state: AppState::default(),
                services,
            },
            Task::perform(async move { load_services.load() }, Message::Loaded),
        )
    })
}

fn update(shell: &mut Shell, message: Message) -> Task<Message> {
    match message {
        Message::Loaded(result) | Message::UsedProfile(result) => {
            match result {
                Ok(loaded) => shell.state.set_loaded(loaded),
                Err(err) => shell.state.set_error(err),
            }

            Task::none()
        }
        Message::ProfileSelected(name) => {
            shell.state.set_selected_profile(name);
            Task::none()
        }
        Message::UseSelectedProfile => {
            let Some(name) = shell.state.selected_profile.clone() else {
                shell.state.set_error("select a profile first".into());
                return Task::none();
            };

            shell.state.is_busy = true;
            shell.state.error_message = None;

            let services = shell.services.clone();
            Task::perform(
                async move { services.use_profile(&name) },
                Message::UsedProfile,
            )
        }
        Message::ApplyPressed => {
            shell.state.is_busy = true;
            shell.state.error_message = None;

            let services = shell.services.clone();
            Task::perform(async move { services.apply() }, Message::Applied)
        }
        Message::Applied(result) | Message::Disabled(result) => {
            match result {
                Ok(action) => shell.state.set_action_result(action),
                Err(err) => shell.state.set_error(err),
            }

            Task::none()
        }
        Message::DisablePressed => {
            shell.state.is_busy = true;
            shell.state.error_message = None;

            let services = shell.services.clone();
            Task::perform(async move { services.disable() }, Message::Disabled)
        }
        Message::CheckInputChanged(value) => {
            shell.state.check_input = value;
            Task::none()
        }
        Message::CheckPressed => {
            let url = shell.state.check_input.trim().to_owned();

            if url.is_empty() {
                shell.state.set_error("enter a URL to check".into());
                return Task::none();
            }

            shell.state.is_busy = true;
            shell.state.error_message = None;

            let services = shell.services.clone();
            Task::perform(async move { services.check(&url) }, Message::Checked)
        }
        Message::Checked(result) => {
            match result {
                Ok(report) => shell.state.set_check_result(report),
                Err(err) => shell.state.set_error(err),
            }

            Task::none()
        }
    }
}

fn view(shell: &Shell) -> Element<'_, Message> {
    let state = &shell.state;

    let summary_section = section(
        "Profile summary",
        column![
            text(format!(
                "Current profile: {}",
                option_text(&state.current_profile)
            )),
            text(format!(
                "Selected profile: {}",
                option_text(&state.selected_profile)
            )),
            text(format!("Mode: {}", text_or_dash(&state.mode_summary))),
            text(format!(
                "Status: {}",
                if state.is_busy { "working" } else { "idle" }
            )),
        ]
        .spacing(8),
    );

    let profiles = if state.profile_names.is_empty() {
        column![text("No profiles found.")].spacing(8)
    } else {
        state
            .profile_names
            .iter()
            .fold(column!().spacing(8), |column, profile| {
                let label = if state.selected_profile.as_deref() == Some(profile.as_str()) {
                    format!("> {}", profile)
                } else {
                    profile.clone()
                };

                column.push(button(text(label)).on_press(Message::ProfileSelected(profile.clone())))
            })
    };

    let use_button = if state.is_busy || state.selected_profile.is_none() {
        button(text("Use selected profile"))
    } else {
        button(text("Use selected profile")).on_press(Message::UseSelectedProfile)
    };

    let profiles_section = section("Profiles", profiles.push(use_button));

    let apply_button = if state.is_busy {
        button(text("Apply"))
    } else {
        button(text("Apply")).on_press(Message::ApplyPressed)
    };
    let disable_button = if state.is_busy {
        button(text("Disable"))
    } else {
        button(text("Disable")).on_press(Message::DisablePressed)
    };

    let mut actions_content = column![row![apply_button, disable_button].spacing(12)].spacing(8);

    if let Some(summary) = &state.action_summary {
        actions_content = actions_content.push(text(format!(
            "{} | success: {} skipped: {} failed: {}",
            summary.message, summary.success_count, summary.skipped_count, summary.failed_count
        )));
    }

    let actions_section = section("Actions", actions_content);

    let check_button = if state.is_busy {
        button(text("Check"))
    } else {
        button(text("Check")).on_press(Message::CheckPressed)
    };

    let check_input = text_input("https://example.com", &state.check_input)
        .on_input(Message::CheckInputChanged)
        .padding(8)
        .size(16);

    let mut check_content = column![row![check_input, check_button].spacing(12)].spacing(8);

    if let Some(report) = &state.last_check {
        check_content = check_content
            .push(text(format!("Target: {}", report.target_url)))
            .push(text(format!("Profile: {}", report.profile_name)))
            .push(text(format!(
                "Transport: {}",
                match &report.transport.value {
                    Some(value) => format!("{:?} ({})", report.transport.mode, value),
                    None => format!("{:?}", report.transport.mode),
                }
            )))
            .push(text(format!("DNS: {}", layer_line(&report.dns))))
            .push(text(format!("TCP: {}", layer_line(&report.tcp))))
            .push(text(format!("TLS: {}", layer_line(&report.tls))))
            .push(text(format!("HTTP: {}", layer_line(&report.http))))
            .push(text(format!("Conclusion: {}", report.conclusion)));
    }

    let check_section = section("Check URL", check_content);

    let mut content = column![
        summary_section,
        profiles_section,
        actions_section,
        check_section
    ]
    .spacing(16)
    .padding(24)
    .width(Length::Fill);

    if let Some(error) = &state.error_message {
        content = content.push(container(text(format!("Error: {}", error))).padding(12));
    }

    container(content)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

fn section<'a>(title: &'a str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(column![text(title).size(20), content.into()].spacing(8))
        .width(Length::Fill)
        .padding(12)
        .into()
}

fn option_text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("none")
}

fn text_or_dash(value: &str) -> &str {
    if value.is_empty() { "-" } else { value }
}

fn layer_line(report: &LayerReport) -> String {
    if report.detail.is_empty() {
        format!("{:?}: {}", report.status, report.summary)
    } else {
        format!(
            "{:?}: {} ({})",
            report.status, report.summary, report.detail
        )
    }
}
