use chrono::Local;
use iced::time::{self, Duration, Instant};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{
    self, Alignment, Application, Command, Element, Length, Settings, Subscription, Theme,
    alignment,
};
// Saved session file name
const DATA_FILE: &str = "sessions.txt";

//To check whether the GUI failed or not
pub fn main() -> iced::Result {
    StudyTimer::run(Settings::default())
}

#[derive(Default)]
struct StudyTimer {
    // App
    screen: Screen,

    // Activity Page
    // - StudyTimer
    is_running: bool,
    mode: Mode,
    duration: Duration,
    remaining: Duration,
    last_tick: Option<Instant>,
    completed: u32,
    interval: u32,
    act_dur: u32,
    short_dur: u32,
    long_dur: u32,

    // - Session Part
    session_active: bool,
    session_started: String,
    session_elapsed: Duration,
    study_elapsed: Duration,
    session_name: String,
    person_name: String,
    // True while the "Save this session?" prompt is shown
    confirming_finish: bool,

    // History Page
    history: Vec<SessionRow>,
    status: String,

    // Settings Page
    act_input: String,
    short_input: String,
    long_input: String,
    interval_input: String,

    light_theme: bool,
}
// Enum for managing screens
#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum Screen {
    #[default]
    Activity,
    History,
    Settings,
}
// Enum for managing timer
#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum Mode {
    #[default]
    Study,
    ShortBreak,
    LongBreak,
}
// Enum for managing settings
#[derive(Debug, Clone, Copy)]
enum Field {
    Activity,
    Short,
    Long,
    Interval,
}

#[derive(Debug, Clone)]
enum Message {
    Open(Screen),
    Tick(Instant),
    Resume,
    Skip,
    Finish,
    ConfirmSave,
    Discard,
    CancelFinish,
    NameChanged(String),
    PersonChanged(String),
    SettingChanged(Field, String),
    ThemeChanged,
}
// Explicit struct for managing each session data
#[derive(Clone, Debug)]
struct SessionRow {
    date: String,
    person: String,
    name: String,
    session_min: u32,
    study_min: u32,
}
// Using application so the program updates automatically regardless of user input
impl Application for StudyTimer {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Theme = Theme;
    type Flags = ();

    //create a new + Default set of time
    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let act_min = 25;
        let short_min = 5;
        let long_min = 15;

        (
            StudyTimer {
                screen: Screen::Activity,
                is_running: false,
                mode: Mode::Study,
                duration: Duration::from_secs((act_min * 60) as u64),
                remaining: Duration::from_secs((act_min * 60) as u64),
                last_tick: None,
                completed: 0,
                interval: 4,
                act_dur: act_min * 60,
                short_dur: short_min * 60,
                long_dur: long_min * 60,
                session_active: false,
                session_started: String::new(),
                session_elapsed: Duration::from_secs(0),
                study_elapsed: Duration::from_secs(0),
                session_name: String::new(),
                person_name: String::new(),
                confirming_finish: false,
                history: load_sessions(),
                status: String::new(),
                act_input: act_min.to_string(),
                short_input: short_min.to_string(),
                long_input: long_min.to_string(),
                interval_input: "4".to_string(),
                light_theme: true,
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            // Switch screen, if switch to history, reloads the sessions
            Message::Open(screen) => {
                if screen == Screen::History {
                    self.history = load_sessions();
                }
                self.screen = screen;
            }
            // Resume the timer
            Message::Resume => {
                if self.is_running {
                    self.is_running = false;
                    self.last_tick = None;
                } else {
                    if !self.session_active {
                        self.begin_session();
                    }
                    if self.remaining.is_zero() {
                        self.complete_mode();
                    }
                    self.is_running = true;
                    self.last_tick = Some(Instant::now());
                }
            }
            //skip = make the app finish counting and record the time
            Message::Skip => self.complete_mode(),
            Message::Finish => {
                // Pause and ask for confirmation
                self.is_running = false;
                self.last_tick = None;
                self.confirming_finish = true;
            }
            Message::ConfirmSave => {
                self.confirming_finish = false;
                self.finish_session(true);
            }
            Message::Discard => {
                self.confirming_finish = false;
                self.finish_session(false);
            }
            Message::CancelFinish => self.confirming_finish = false,
            //Set timer logic
            Message::Tick(now) => {
                if self.is_running {
                    if let Some(last) = self.last_tick {
                        let elapsed = now.saturating_duration_since(last);
                        let used = elapsed.min(self.remaining);
                        self.remaining = self.remaining.saturating_sub(used);
                        self.session_elapsed += used;
                        if self.mode == Mode::Study {
                            self.study_elapsed += used;
                        }
                    }
                    self.last_tick = Some(now);
                    if self.remaining.is_zero() {
                        self.complete_mode();
                    }
                }
            }
            Message::NameChanged(value) => self.session_name = value,
            Message::PersonChanged(value) => self.person_name = value,
            Message::SettingChanged(field, value) => self.change_setting(field, value),
            Message::ThemeChanged => self.light_theme = !self.light_theme,
        }
        Command::none()
    }

    fn title(&self) -> String {
        "Pomodoro Technique Study App".to_string()
    }

    fn view(&self) -> Element<Self::Message> {
        let body = match self.screen {
            Screen::Activity => self.view_activity(),
            Screen::History => self.view_history(),
            Screen::Settings => self.view_settings(),
        };

        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.is_running {
            time::every(Duration::from_millis(200)).map(Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn theme(&self) -> Self::Theme {
        if self.light_theme {
            Theme::Light
        } else {
            Theme::Dark
        }
    }
}

impl StudyTimer {
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        let new_duration = match mode {
            Mode::Study => self.act_dur,
            Mode::ShortBreak => self.short_dur,
            Mode::LongBreak => self.long_dur,
        };
        self.duration = Duration::from_secs(new_duration as u64);
        self.remaining = self.duration;
    }

    fn complete_mode(&mut self) {
        match self.mode {
            Mode::Study => {
                self.completed += 1;
                if self.completed % self.interval == 0 {
                    self.set_mode(Mode::LongBreak);
                } else {
                    self.set_mode(Mode::ShortBreak);
                }
            }
            Mode::ShortBreak | Mode::LongBreak => {
                self.set_mode(Mode::Study);
            }
        }
    }

    fn change_setting(&mut self, field: Field, value: String) {
        let digits: String = value
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(3)
            .collect();

        match field {
            Field::Activity => {
                self.act_input = digits.clone();
                if let Ok(n) = digits.parse::<u32>() {
                    self.act_dur = n.clamp(1, 180) * 60;
                }
            }
            Field::Short => {
                self.short_input = digits.clone();
                if let Ok(n) = digits.parse::<u32>() {
                    self.short_dur = n.clamp(1, 60) * 60;
                }
            }
            Field::Long => {
                self.long_input = digits.clone();
                if let Ok(n) = digits.parse::<u32>() {
                    self.long_dur = n.clamp(1, 120) * 60;
                }
            }
            Field::Interval => {
                self.interval_input = digits.clone();
                if let Ok(n) = digits.parse::<u32>() {
                    self.interval = n.clamp(1, 12);
                }
            }
        }

        if !self.session_active {
            let mode = self.mode;
            self.set_mode(mode);
        }
    }

    fn view_activity(&self) -> Element<<StudyTimer as iced::Application>::Message> {
        let phase_label = |msg: &str, active| {
            let label = if active {
                format!("> {} <", msg)
            } else {
                msg.to_string()
            };
            container(
                text(label)
                    .width(Length::Fixed(160.0))
                    .height(Length::Fixed(42.0))
                    .size(20)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
        };

        let phases = row![
            phase_label("Study", self.mode == Mode::Study),
            phase_label("Short Break", self.mode == Mode::ShortBreak),
            phase_label("Long Break", self.mode == Mode::LongBreak)
        ]
        .spacing(16)
        .width(Length::Shrink)
        .align_items(Alignment::Center);

        let secs = if self.remaining.is_zero() {
            0
        } else {
            self.remaining.as_secs() + u64::from(self.remaining.subsec_nanos() > 0)
        };

        let clock = container(
            text(format!("{:02}:{:02}", secs / 60, secs % 60))
                .size(96.0)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .center_x();

        let start_message = if !self.is_running { "Start" } else { "Pause" };

        let controls = row![
            button(
                text(start_message)
                    .size(28)
                    .horizontal_alignment(alignment::Horizontal::Center)
            )
            .width(Length::Fixed(160.0))
            .height(Length::Fixed(58.0))
            .on_press(Message::Resume),
            button(
                text("Skip")
                    .size(28)
                    .horizontal_alignment(alignment::Horizontal::Center)
            )
            .width(Length::Fixed(160.0))
            .height(Length::Fixed(58.0))
            .on_press(Message::Skip)
        ]
        .spacing(16);

        let mode_text = match self.mode {
            Mode::Study => "Study",
            Mode::ShortBreak => "Short Break",
            Mode::LongBreak => "Long Break",
        };

        let top_panel = container(
            column![
                phases,
                text(format!("Mode: {}", mode_text)),
                clock,
                controls
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(20);

        let navs = row![
            button(
                text("History")
                    .size(28)
                    .horizontal_alignment(alignment::Horizontal::Center)
            )
            .width(Length::Fixed(160.0))
            .height(Length::Fixed(58.0))
            .on_press(Message::Open(Screen::History)),
            button(
                text("Setting")
                    .size(28)
                    .horizontal_alignment(alignment::Horizontal::Center)
            )
            .width(Length::Fixed(160.0))
            .height(Length::Fixed(58.0))
            .on_press(Message::Open(Screen::Settings))
        ]
        .spacing(16)
        .width(Length::Shrink);

        let bottom_panel = container(
            column![
                row![
                    container(text("Session :")).width(Length::Fixed(110.0)),
                    text_input("Session name", &self.session_name).on_input(Message::NameChanged)
                ]
                .width(Length::Fixed(520.0)),
                row![
                    container(text("Name :")).width(Length::Fixed(110.0)),
                    text_input("Your name", &self.person_name).on_input(Message::PersonChanged)
                ]
                .width(Length::Fixed(520.0)),
            ]
            .spacing(10),
        )
        .width(Length::Shrink)
        .padding(20);

        let mut content = column![top_panel, navs, bottom_panel]
            .spacing(22)
            .width(Length::Fill)
            .align_items(Alignment::Center);

        if self.session_active {
            if self.confirming_finish {
                let small = |label: &str, msg: Message| {
                    button(
                        text(label.to_string())
                            .size(22)
                            .horizontal_alignment(alignment::Horizontal::Center),
                    )
                    .width(Length::Fixed(130.0))
                    .height(Length::Fixed(46.0))
                    .on_press(msg)
                };

                content = content.push(text("Save this session?").size(24));
                content = content.push(
                    row![
                        small("Save", Message::ConfirmSave),
                        small("Discard", Message::Discard),
                        small("Cancel", Message::CancelFinish),
                    ]
                    .spacing(12),
                );
            } else {
                content = content.push(
                    button(
                        text("Finish")
                            .size(26)
                            .horizontal_alignment(alignment::Horizontal::Center),
                    )
                    .width(Length::Fixed(180.0))
                    .height(Length::Fixed(52.0))
                    .on_press(Message::Finish),
                );
            }
        }

        if !self.status.is_empty() {
            content = content.push(text(self.status.as_str()).size(16));
        }

        container(content).into()
    }

    fn view_history(&self) -> Element<<StudyTimer as iced::Application>::Message> {
        let top = row![
            text("History").size(40),
            container(button(text("Back").size(24)).on_press(Message::Open(Screen::Activity)))
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Right)
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill);

        let header = row![
            container(text("Date").size(20)).width(Length::FillPortion(2)),
            container(text("Session / Name").size(20)).width(Length::FillPortion(4)),
            container(text("Session / Study").size(20)).width(Length::FillPortion(3)),
        ]
        .spacing(8)
        .width(Length::Fill);

        let mut items = column![].spacing(8).width(Length::Fill);
        if self.history.is_empty() {
            items = items.push(text("No sessions yet."));
        } else {
            for row_data in self.history.iter().rev() {
                items = items.push(
                    row![
                        container(text(short_date(&row_data.date))).width(Length::FillPortion(2)),
                        container(text(format!("{} / {}", row_data.name, row_data.person)))
                            .width(Length::FillPortion(4)),
                        container(text(format!(
                            "{} / {}",
                            format_minutes(row_data.session_min),
                            format_minutes(row_data.study_min)
                        )))
                        .width(Length::FillPortion(3)),
                    ]
                    .spacing(8)
                    .width(Length::Fill),
                );
            }
        }

        let body = column![top, header, scrollable(items).height(Length::Fill)]
            .spacing(14)
            .width(Length::Fill)
            .height(Length::Fill);

        container(body).padding(20).into()
    }

    fn view_settings(&self) -> Element<<StudyTimer as iced::Application>::Message> {
        let top = row![
            text("Setting").size(40),
            container(button(text("Back").size(24)).on_press(Message::Open(Screen::Activity)))
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Right)
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill);

        let theme_label = if self.light_theme { "Light" } else { "Dark" };

        let fields = column![
            row![
                container(text("Activity (min):")).width(Length::Fixed(190.0)),
                text_input("25", &self.act_input)
                    .width(Length::Fixed(100.0))
                    .on_input(|v| Message::SettingChanged(Field::Activity, v)),
            ]
            .spacing(10),
            row![
                container(text("Short break (min):")).width(Length::Fixed(190.0)),
                text_input("5", &self.short_input)
                    .width(Length::Fixed(100.0))
                    .on_input(|v| Message::SettingChanged(Field::Short, v)),
            ]
            .spacing(10),
            row![
                container(text("Long break (min):")).width(Length::Fixed(190.0)),
                text_input("15", &self.long_input)
                    .width(Length::Fixed(100.0))
                    .on_input(|v| Message::SettingChanged(Field::Long, v)),
            ]
            .spacing(10),
            row![
                container(text("Long break every:")).width(Length::Fixed(190.0)),
                text_input("4", &self.interval_input)
                    .width(Length::Fixed(100.0))
                    .on_input(|v| Message::SettingChanged(Field::Interval, v)),
            ]
            .spacing(10),
            row![
                container(text("Theme:")).width(Length::Fixed(190.0)),
                button(theme_label)
                    .width(Length::Fixed(100.0))
                    .on_press(Message::ThemeChanged),
            ]
            .spacing(10),
        ]
        .spacing(12);

        let panel = column![
            top,
            fields,
            text("Changes apply immediately.").size(16),
            text("Current mode length updates only when no session is active.").size(14),
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Start);

        container(panel).padding(20).into()
    }

    // `save` = true writes the session to the file, false throws it away
    fn finish_session(&mut self, save: bool) {
        self.is_running = false;
        self.last_tick = None;

        if !save {
            self.status = "Session discarded.".to_string();
        } else if self.session_active && self.session_elapsed.as_secs() >= 1 {
            let row = SessionRow {
                date: self.session_started.clone(),
                person: clean(&self.person_name),
                name: clean(&self.session_name),
                session_min: to_minutes(self.session_elapsed),
                study_min: to_minutes(self.study_elapsed),
            };
            self.status = match append_session(&row) {
                Ok(()) => format!("Session saved to {}.", DATA_FILE),
                Err(e) => format!("Save failed: {}", e),
            };
            self.history = load_sessions();
        } else {
            self.status = "Nothing to save yet.".to_string();
        }

        self.session_active = false;
        self.session_elapsed = Duration::from_secs(0);
        self.study_elapsed = Duration::from_secs(0);
        self.completed = 0;
        self.set_mode(Mode::Study);
    }

    fn begin_session(&mut self) {
        self.session_active = true;
        self.session_started = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.session_elapsed = Duration::from_secs(0);
        self.study_elapsed = Duration::from_secs(0);
        self.completed = 0;
        self.status.clear();
        self.set_mode(Mode::Study);
    }
}

// Clean user text before saving it:
// replace '|' and line breaks with spaces (because '|' separates columns in the file),
// trim spaces, and use "Unknown" if nothing is left
fn clean(value: &str) -> String {
    let v = value.replace(['|', '\n', '\r'], " ");
    let v = v.trim();
    if v.is_empty() {
        "Unknown".to_string()
    } else {
        v.to_string()
    }
}

fn append_session(row: &SessionRow) -> std::io::Result<()> {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(DATA_FILE)?;

    writeln!(
        file,
        "{} | {} | {} | {} | {}",
        row.date, row.person, row.name, row.session_min, row.study_min
    )
}

fn load_sessions() -> Vec<SessionRow> {
    let content = std::fs::read_to_string(DATA_FILE).unwrap_or_default();
    let mut rows = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(5, '|').map(str::trim).collect();
        if parts.len() < 4 {
            continue;
        }

        let (session_min, study_min) = if parts.len() == 5 {
            match (parts[3].parse::<u32>(), parts[4].parse::<u32>()) {
                (Ok(a), Ok(b)) => (a, b),
                _ => continue,
            }
        } else {
            match parts[3].parse::<u32>() {
                Ok(a) => (a, a),
                Err(_) => continue,
            }
        };

        rows.push(SessionRow {
            date: parts[0].to_string(),
            person: parts[1].to_string(),
            name: parts[2].to_string(),
            session_min,
            study_min,
        });
    }

    rows
}

fn to_minutes(d: Duration) -> u32 {
    ((d.as_secs() + 30) / 60) as u32
}

fn short_date(date: &str) -> String {
    match (date.get(2..4), date.get(5..7), date.get(8..10)) {
        (Some(y), Some(m), Some(d)) => format!("{d}/{m}/{y}"),
        _ => date.to_string(),
    }
}

fn format_minutes(minutes: u32) -> String {
    format!("{}h {}m", minutes / 60, minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Application;

    fn new_app() -> StudyTimer {
        let (app, _cmd) = <StudyTimer as Application>::new(());
        app
    }

    #[test]
    fn test_clean() {
        assert_eq!(clean("  Alice|Bob\n"), "Alice Bob");
        assert_eq!(clean(" \r\n "), "Unknown");
    }

    #[test]
    fn test_to_minutes() {
        assert_eq!(to_minutes(Duration::from_secs(29)), 0);
        assert_eq!(to_minutes(Duration::from_secs(30)), 1);
        assert_eq!(to_minutes(Duration::from_secs(89)), 1);
        assert_eq!(to_minutes(Duration::from_secs(90)), 2);
    }

    #[test]
    fn test_short_date() {
        assert_eq!(short_date("2026-11-15 10:23:00"), "15/11/26");
    }

    #[test]
    fn test_format_minutes() {
        assert_eq!(format_minutes(0), "0h 0m");
        assert_eq!(format_minutes(61), "1h 1m");
        assert_eq!(format_minutes(135), "2h 15m");
    }

    #[test]
    fn test_change_setting() {
        let mut app = new_app();

        app.change_setting(Field::Activity, "999".to_string());
        assert_eq!(app.act_dur, 180 * 60);
        assert_eq!(app.remaining, Duration::from_secs((180 * 60) as u64));

        app.change_setting(Field::Interval, "13".to_string());
        assert_eq!(app.interval, 12);
    }
}
