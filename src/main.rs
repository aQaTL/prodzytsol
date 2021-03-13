use anyhow::Result;
use iced::window::Mode;
use iced::*;
use iced_native::event::Status;
use iced_native::Event;
use log::*;

mod commands;
mod views;

fn main() -> Result<()> {
    flexi_logger::Logger::with_env_or_str("presentation=debug").start()?;
    App::run(Settings::default())?;

    Ok(())
}

pub struct App {
    stage: Stage,
    mode: Mode,
}

pub enum Stage {
    WelcomeScreen,
    Presentation {
        presentation: Presentation,
        state: PresentationState,
    },
}

#[derive(Debug)]
pub struct Presentation {
    title: String,
    slides: Vec<Slide>,
}

#[derive(Debug, Default)]
pub struct PresentationState {
    slide_idx: usize,
}

#[derive(Debug, Default)]
pub struct Slide {
    header: Option<String>,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app = App {
            stage: Stage::WelcomeScreen,
            mode: Mode::Windowed,
        };
        let command = Command::perform(commands::load_from_args(), Message::Loaded);
        (app, command)
    }

    fn title(&self) -> String {
        match self.stage {
            Stage::WelcomeScreen => String::from("Presentation"),
            Stage::Presentation {
                ref presentation, ..
            } => presentation.title.clone(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Loaded(Ok(presentation)) => {
                info!("Loaded presentation \"{}\"", presentation.title);
                self.stage = Stage::Presentation {
                    presentation,
                    state: Default::default(),
                };
            }
            Message::Loaded(Err(e)) => {
                error!("Failed to load presentation: {:?}", e);
            }
            Message::KeyboardEvent(e) => return self.handle_keyboard_event(e),
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        match self.stage {
            Stage::WelcomeScreen => views::welcome_screen(),
            Stage::Presentation {
                ref presentation,
                ref state,
            } => views::presentation(presentation, state),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events_with(|ev, status| match (ev, status) {
            (_, Status::Captured) => None,
            (Event::Keyboard(e), Status::Ignored) => Some(Message::KeyboardEvent(e)),
            (_, Status::Ignored) => None,
        })
    }

    fn mode(&self) -> Mode {
        self.mode
    }

    fn background_color(&self) -> Color {
        Color::from_rgb8(0, 0, 0)
    }

    // fn scale_factor(&self) -> f64 { }
}

impl App {
    fn handle_keyboard_event(&mut self, kb_ev: keyboard::Event) -> Command<Message> {
        use keyboard::{Event::*, KeyCode};
        match kb_ev {
            KeyPressed {
                key_code: KeyCode::Space,
                modifiers: _,
            } => {
                if let Stage::Presentation {
                    ref presentation,
                    ref mut state,
                } = self.stage
                {
                    state.slide_idx = (state.slide_idx + 1).min(presentation.slides.len() - 1);
                    info!("new slide idx: {}", state.slide_idx);
                }
            }

            _ => (),
        }
        Command::none()
    }
}

#[derive(Debug)]
pub enum Message {
    Loaded(commands::LoadFromArgsResult),
    KeyboardEvent(keyboard::Event),
}
