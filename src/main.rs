use anyhow::Result;
use iced::window::Mode;
use iced::*;
use iced_futures::BoxStream;
use iced_native::event::Status;
use iced_native::Event;
use log::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

mod commands;
mod parser;
mod views;

fn main() -> Result<()> {
	flexi_logger::Logger::with_env_or_str(concat!(env!("CARGO_PKG_NAME"), "=debug")).start()?;
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
		file_watcher: Option<FileWatch>,
	},
}

#[derive(Debug)]
pub struct Presentation {
	title: String,
	path: PathBuf,
	slides: Vec<Slide>,
}

#[derive(Debug, Default, Clone)]
pub struct PresentationState {
	slide_idx: usize,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Slide {
	nodes: Vec<SlideNode>,
	background: Option<Image>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SlideNode {
	Header(HeaderSize, String),
	Text(String),
	UnnumberedList(Vec<String>),
	NumberedList(Vec<String>),
	Image(Image),
	CodeBlock(Language, String),
}

#[derive(Debug)]
pub struct Image {
	path: String,
	alt_text: String,
	handle: Option<image::Handle>,
}

impl PartialEq for Image {
	fn eq(&self, other: &Self) -> bool {
		self.path == other.path && self.alt_text == other.alt_text
	}
}

impl Eq for Image {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Language {
	Rust,
}

impl FromStr for Language {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use Language::*;
		Ok(match s {
			"rust" => Rust,
			_ => anyhow::bail!("Unknown language {}", s),
		})
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum HeaderSize {
	One = 1,
	Two,
	Three,
	Four,
	Five,
}

impl HeaderSize {
	pub fn to_font_size(self) -> u16 {
		let size = self as u8 as f32;
		(100.0 * (1.0 - (size * 10.0 * 0.01))) as u16
	}

	unsafe fn from_u8_unchecked(n: u8) -> Self {
		std::mem::transmute(n)
	}
}

#[test]
fn header_size_test() {
	let hs1 = HeaderSize::One;
	let hs2 = HeaderSize::Two;
	let hs5 = HeaderSize::Five;

	assert_eq!(hs1 as u8, 1);
	assert_eq!(hs2 as u8, 2);
	assert_eq!(hs5 as u8, 5);
}

#[test]
fn header_size_to_font_size_conversion_test() {
	assert_eq!(HeaderSize::One.to_font_size(), 90);
	assert_eq!(HeaderSize::Two.to_font_size(), 80);
	assert_eq!(HeaderSize::Three.to_font_size(), 70);
	assert_eq!(HeaderSize::Four.to_font_size(), 60);
	assert_eq!(HeaderSize::Five.to_font_size(), 50);
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

	fn update(
		&mut self,
		message: Self::Message,
		_clipboard: &mut Clipboard,
	) -> Command<Self::Message> {
		match message {
			Message::Loaded(Ok(presentation)) => {
				info!("Loaded presentation \"{}\"", presentation.title);

				let (state, file_watcher) = match self.stage {
					Stage::Presentation {
						ref state,
						ref mut file_watcher,
						..
					} => (state.clone(), file_watcher.take()),
					_ => (PresentationState::default(), None),
				};

				let cmd = if file_watcher.is_none() {
					Command::perform(
						commands::start_file_watcher(presentation.path.clone()),
						Message::FileWatcherStarted,
					)
				} else {
					Command::none()
				};

				self.stage = Stage::Presentation {
					presentation,
					state,
					file_watcher,
				};

				return cmd;
			}
			Message::Loaded(Err(e)) => {
				error!("Failed to load presentation: {:?}", e);
			}
			Message::FileWatcherStarted(Some(new_file_watcher)) => {
				if let Stage::Presentation {
					ref mut file_watcher,
					..
				} = self.stage
				{
					*file_watcher = Some(new_file_watcher);
				}
			}
			Message::FileWatcherStarted(None) => (),
			Message::Reloaded => {
				info!("Presentation file has been updated. Reloading");
				return Command::perform(commands::load_from_args(), Message::Loaded);
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
				..
			} => views::presentation(presentation, state),
		}
	}

	fn subscription(&self) -> Subscription<Self::Message> {
		let mut subscriptions = Vec::new();
		if let Stage::Presentation {
			file_watcher: Some(file_watcher),
			..
		} = &self.stage
		{
			let sub =
				iced::Subscription::from_recipe(file_watcher.recipe()).map(|_| Message::Reloaded);
			subscriptions.push(sub);
		}
		let sub = iced_native::subscription::events_with(|ev, status| match (ev, status) {
			(_, Status::Captured) => None,
			(Event::Keyboard(e), Status::Ignored) => Some(Message::KeyboardEvent(e)),
			(_, Status::Ignored) => None,
		});
		subscriptions.push(sub);
		Subscription::batch(subscriptions)
	}

	fn mode(&self) -> Mode {
		self.mode
	}

	fn background_color(&self) -> Color {
		Color::from_rgb8(0, 0, 0)
	}

	// fn scale_factor(&self) -> f64 { }
}

pub struct FileWatch {
	#[allow(dead_code)]
	watcher: RecommendedWatcher,
	sender: tokio::sync::broadcast::Sender<()>,
	id: usize,
}

impl Debug for FileWatch {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("FileWatch").field("id", &self.id).finish()
	}
}

impl FileWatch {
	pub async fn new(path: PathBuf) -> Result<Self> {
		info!("Setting up file watcher for {}", path.display());
		let (sender, _) = tokio::sync::broadcast::channel(10);
		let sender_2 = sender.clone();

		let watcher = RecommendedWatcher::new(move |res| match res {
			Ok(notify::Event {
				kind: notify::EventKind::Modify(_),
				..
			}) => {
				// info!("Sending watcher update");
				if sender_2.send(()).is_err() {
					error!("File watcher received end has been dropped");
				}
			}
			Err(e) => {
				error!("File watcher error: {:?}", e);
			}
			_ => (),
		});

		let mut watcher = match watcher {
			Ok(v) => v,
			Err(e) => {
				anyhow::bail!("Failed to create file watcher: {:?}", e);
			}
		};

		if let Err(e) = watcher.watch(&path, RecursiveMode::Recursive) {
			anyhow::bail!("Failed to create file watcher: {:?}", e);
		}

		// let _ = receiver.changed().await?;
		info!("File watcher set up successfully");

		Ok(FileWatch {
			watcher,
			sender,
			id: rand::random(),
		})
	}

	pub fn recipe(&self) -> FileWatchRecipe {
		FileWatchRecipe(self.id, self.sender.clone())
	}
}

pub struct FileWatchRecipe(usize, tokio::sync::broadcast::Sender<()>);

impl<H, E> iced_futures::subscription::Recipe<H, E> for FileWatchRecipe
where
	H: std::hash::Hasher,
{
	type Output = ();

	fn hash(&self, state: &mut H) {
		use std::hash::Hash;
		std::any::TypeId::of::<Self>().hash(state);
		self.0.hash(state);
	}

	fn stream(self: Box<Self>, _input: BoxStream<E>) -> BoxStream<Self::Output> {
		info!("Creating new FileWatchRecipe stream");
		Box::pin(
			BroadcastStream::new(self.1.subscribe())
				.take_while(|x| x.is_ok())
				.fuse()
				.map(|x| x.unwrap()),
		)
	}
}

impl App {
	fn handle_keyboard_event(&mut self, kb_ev: keyboard::Event) -> Command<Message> {
		use keyboard::{Event::*, KeyCode};
		match kb_ev {
			KeyPressed {
				key_code: KeyCode::Space,
				modifiers: _,
			}
			| KeyPressed {
				key_code: KeyCode::X,
				modifiers: _,
			}
			| KeyPressed {
				key_code: KeyCode::Right,
				modifiers: _,
			} => {
				if let Stage::Presentation {
					ref presentation,
					ref mut state,
					..
				} = self.stage
				{
					state.slide_idx = (state.slide_idx + 1).min(presentation.slides.len() - 1);
				}
			}

			KeyPressed {
				key_code: KeyCode::Backspace,
				modifiers: _,
			}
			| KeyPressed {
				key_code: KeyCode::Z,
				modifiers: _,
			}
			| KeyPressed {
				key_code: KeyCode::Left,
				modifiers: _,
			} => {
				if let Stage::Presentation {
					presentation: _,
					ref mut state,
					..
				} = self.stage
				{
					state.slide_idx = state.slide_idx.saturating_sub(1);
				}
			}

			KeyPressed {
				key_code: KeyCode::F,
				modifiers: _,
			} => match self.mode {
				ref mut mode @ Mode::Windowed => *mode = Mode::Fullscreen,
				ref mut mode @ Mode::Fullscreen => *mode = Mode::Windowed,
				Mode::Hidden => (),
			},

			_ => (),
		}
		Command::none()
	}
}

#[derive(Debug)]
pub enum Message {
	Loaded(commands::LoadFromArgsResult),
	FileWatcherStarted(commands::StartFileWatcherResult),
	Reloaded,
	KeyboardEvent(keyboard::Event),
}
