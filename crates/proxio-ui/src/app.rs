use cosmic::app::{Core, Task};
use cosmic::iced::widget::{column, container, row};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, settings, warning};
use cosmic::{ApplicationExt, Element, executor};
use proxio_diagnose::{CheckReport, LayerReport};

use crate::services::{ActionResult, LoadedState, RealServices, UiServices};

#[derive(Debug, Clone, Default)]
pub struct ActionSummary {
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosisRow {
    pub layer: String,
    pub status: String,
    pub summary: String,
    pub detail: Option<String>,
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
    pub diagnosis_rows: Vec<DiagnosisRow>,
    pub is_busy: bool,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn from_loaded(
        profile_names: Vec<String>,
        current_profile: Option<String>,
        mode_summary: String,
    ) -> Self {
        let mut state = Self {
            profile_names,
            selected_profile: current_profile.clone(),
            ..Self::default()
        };
        state.set_header_summary(current_profile, mode_summary);
        state
    }

    pub fn set_header_summary(&mut self, profile: Option<String>, mode_summary: String) {
        self.current_profile = profile;
        self.mode_summary = mode_summary;
    }

    pub fn set_selected_profile(&mut self, name: String) {
        self.selected_profile = Some(name);
        self.clear_error();
    }

    pub fn set_loaded(&mut self, loaded: LoadedState) {
        self.profile_names = loaded.profile_names;
        self.selected_profile = loaded.current_profile.clone();
        self.set_header_summary(loaded.current_profile, loaded.mode_summary);
        self.is_busy = false;
        self.clear_error();
    }

    pub fn set_action_result(&mut self, result: ActionResult) {
        self.action_summary = Some(ActionSummary {
            success_count: result.success_count,
            skipped_count: result.skipped_count,
            failed_count: result.failed_count,
            message: result.message,
        });
        self.is_busy = false;
        self.clear_error();
    }

    pub fn set_check_result(&mut self, report: CheckReport) {
        self.diagnosis_rows = diagnosis_rows(&report);
        self.last_check = Some(report);
        self.is_busy = false;
        self.clear_error();
    }

    pub fn set_diagnosis_rows(&mut self, rows: Vec<DiagnosisRow>) {
        self.diagnosis_rows = rows;
    }

    pub fn clear_error(&mut self) {
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
    ClearError,
}

pub struct App {
    core: Core,
    state: AppState,
    services: Option<RealServices>,
}

pub fn run() -> cosmic::iced::Result {
    let settings = cosmic::app::Settings::default();
    cosmic::app::run::<App>(settings, ())
}

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.proxio.ui";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut app = Self {
            core,
            state: AppState::default(),
            services: None,
        };

        let mut startup = app.set_titles();

        match RealServices::new() {
            Ok(services) => {
                startup = Task::batch(vec![startup, load_task(services.clone())]);
                app.services = Some(services);
            }
            Err(err) => app.state.set_error(err),
        }

        (app, startup)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Loaded(result) | Message::UsedProfile(result) => match result {
                Ok(loaded) => {
                    self.state.set_loaded(loaded);
                    self.set_titles()
                }
                Err(err) => {
                    self.state.set_error(err);
                    Task::none()
                }
            },
            Message::ProfileSelected(name) => {
                self.state.set_selected_profile(name);
                Task::none()
            }
            Message::UseSelectedProfile => {
                let Some(name) = self.state.selected_profile.clone() else {
                    self.state.set_error("select a profile first".into());
                    return Task::none();
                };

                let Some(services) = self.services.clone() else {
                    self.state.set_error("services are unavailable".into());
                    return Task::none();
                };

                self.state.is_busy = true;
                self.state.clear_error();
                cosmic::task::future(async move { Message::UsedProfile(services.use_profile(&name)) })
            }
            Message::ApplyPressed => {
                let Some(services) = self.services.clone() else {
                    self.state.set_error("services are unavailable".into());
                    return Task::none();
                };

                self.state.is_busy = true;
                self.state.clear_error();
                cosmic::task::future(async move { Message::Applied(services.apply()) })
            }
            Message::Applied(result) | Message::Disabled(result) => {
                match result {
                    Ok(action) => self.state.set_action_result(action),
                    Err(err) => self.state.set_error(err),
                }

                Task::none()
            }
            Message::DisablePressed => {
                let Some(services) = self.services.clone() else {
                    self.state.set_error("services are unavailable".into());
                    return Task::none();
                };

                self.state.is_busy = true;
                self.state.clear_error();
                cosmic::task::future(async move { Message::Disabled(services.disable()) })
            }
            Message::CheckInputChanged(value) => {
                self.state.check_input = value;
                Task::none()
            }
            Message::CheckPressed => {
                let url = self.state.check_input.trim().to_owned();
                if url.is_empty() {
                    self.state.set_error("enter a URL to check".into());
                    return Task::none();
                }

                let Some(services) = self.services.clone() else {
                    self.state.set_error("services are unavailable".into());
                    return Task::none();
                };

                self.state.is_busy = true;
                self.state.clear_error();
                cosmic::task::future(async move { Message::Checked(services.check(&url)) })
            }
            Message::Checked(result) => {
                match result {
                    Ok(report) => self.state.set_check_result(report),
                    Err(err) => self.state.set_error(err),
                }

                Task::none()
            }
            Message::ClearError => {
                self.state.clear_error();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let state = &self.state;

        let header = settings::section()
            .title("Overview")
            .add(settings::item(
                "Active profile",
                widget::text::body(option_text(&state.current_profile)),
            ))
            .add(settings::item(
                "Selected profile",
                widget::text::body(option_text(&state.selected_profile)),
            ))
            .add(settings::item(
                "Proxy mode",
                widget::text::body(text_or_dash(&state.mode_summary)),
            ))
            .add(settings::item(
                "Status",
                widget::text::body(if state.is_busy { "Working" } else { "Idle" }),
            ));

        let profiles = settings::section()
            .title("Profiles")
            .add(profile_list(state))
            .add(settings::item(
                "Current selection",
                button::suggested("Use profile")
                    .on_press_maybe((!state.is_busy && state.selected_profile.is_some()).then_some(
                        Message::UseSelectedProfile,
                    )),
            ));

        let actions = settings::section()
            .title("Actions")
            .add(settings::item(
                "Apply current configuration",
                action_buttons(state),
            ))
            .add_maybe(state.action_summary.as_ref().map(action_summary_view));

        let diagnosis = settings::section()
            .title("Connection check")
            .add(settings::item(
                "Target URL",
                widget::text_input::text_input("https://example.com", &state.check_input)
                    .on_input(Message::CheckInputChanged)
                    .width(Length::Fixed(280.0)),
            ))
            .add(settings::item(
                "Run layered check",
                button::standard("Check")
                    .on_press_maybe((!state.is_busy).then_some(Message::CheckPressed)),
            ))
            .add_maybe(state.last_check.as_ref().map(check_summary_view))
            .extend(state.diagnosis_rows.iter().map(diagnosis_row_view));

        let top = row![
            container(profiles).width(Length::FillPortion(2)),
            container(actions).width(Length::FillPortion(3))
        ]
        .spacing(24)
        .align_y(Alignment::Start);

        let mut content = settings::view_column(vec![
            header.into(),
            top.into(),
            diagnosis.into(),
        ])
        .padding(24)
        .width(Length::Fill);

        if let Some(error) = &state.error_message {
            content = content.push(warning(error).on_close(Message::ClearError));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }
}

impl App {
    fn set_titles(&mut self) -> Task<Message> {
        let active = option_text(&self.state.current_profile);
        let mode = text_or_dash(&self.state.mode_summary);
        let title = format!("Proxio: {active} ({mode})");
        self.set_header_title(title.clone());
        self.set_window_title(title)
    }
}

fn load_task(services: RealServices) -> Task<Message> {
    cosmic::task::future(async move { Message::Loaded(services.load()) })
}

fn profile_list(state: &AppState) -> Element<'_, Message> {
    if state.profile_names.is_empty() {
        return widget::text::body("No profiles found.").into();
    }

    state
        .profile_names
        .iter()
        .fold(widget::column::with_capacity(state.profile_names.len()).spacing(8), |column, profile| {
            let label = if state.selected_profile.as_deref() == Some(profile.as_str()) {
                format!("Selected: {profile}")
            } else if state.current_profile.as_deref() == Some(profile.as_str()) {
                format!("{profile} (active)")
            } else {
                profile.clone()
            };

            column.push(
                button::standard(label)
                    .width(Length::Fill)
                    .on_press_maybe((!state.is_busy).then_some(Message::ProfileSelected(profile.clone()))),
            )
        })
        .into()
}

fn action_buttons(state: &AppState) -> Element<'_, Message> {
    row![
        button::suggested("Apply")
            .on_press_maybe((!state.is_busy).then_some(Message::ApplyPressed)),
        button::destructive("Disable")
            .on_press_maybe((!state.is_busy).then_some(Message::DisablePressed)),
    ]
    .spacing(12)
    .into()
}

fn action_summary_view(summary: &ActionSummary) -> Element<'_, Message> {
    settings::item_row(vec![
        summary_card("Message", &summary.message),
        summary_card("Success", &summary.success_count.to_string()),
        summary_card("Skipped", &summary.skipped_count.to_string()),
        summary_card("Failed", &summary.failed_count.to_string()),
    ])
    .into()
}

fn check_summary_view(report: &CheckReport) -> Element<'_, Message> {
    let transport = match &report.transport.value {
        Some(value) => format!("{:?} via {}", report.transport.mode, value),
        None => format!("{:?}", report.transport.mode),
    };

    settings::section()
        .title("Last report")
        .add(settings::item(
            "Profile",
            widget::text::body(report.profile_name.as_str()),
        ))
        .add(settings::item(
            "Transport",
            widget::text::body(transport),
        ))
        .add(settings::item(
            "Conclusion",
            widget::text::body(report.conclusion.as_str()),
        ))
        .into()
}

fn diagnosis_row_view(row_state: &DiagnosisRow) -> Element<'_, Message> {
    let detail = row_state.detail.as_deref().unwrap_or("No extra detail");

    settings::section()
        .title(row_state.layer.clone())
        .add(settings::item(
            "Status",
            widget::text::body(row_state.status.clone()),
        ))
        .add(settings::item(
            "Summary",
            widget::text::body(row_state.summary.clone()),
        ))
        .add(settings::item("Detail", widget::text::body(detail)))
        .into()
}

fn summary_card<'a>(title: impl Into<String>, value: impl Into<String>) -> Element<'a, Message> {
    let title = title.into();
    let value = value.into();

    container(
        column![
            widget::text::caption(title),
            widget::text::title4(value),
        ]
        .spacing(4),
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn diagnosis_rows(report: &CheckReport) -> Vec<DiagnosisRow> {
    vec![
        diagnosis_row("DNS", &report.dns),
        diagnosis_row("TCP", &report.tcp),
        diagnosis_row("TLS", &report.tls),
        diagnosis_row("HTTP", &report.http),
    ]
}

fn diagnosis_row(layer: &str, report: &LayerReport) -> DiagnosisRow {
    DiagnosisRow {
        layer: layer.to_owned(),
        status: format!("{:?}", report.status),
        summary: report.summary.clone(),
        detail: (!report.detail.is_empty()).then(|| report.detail.clone()),
    }
}

fn option_text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("none")
}

fn text_or_dash(value: &str) -> &str {
    if value.is_empty() {
        "-"
    } else {
        value
    }
}
