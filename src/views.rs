use crate::{
	App, CodeBlockParams, CodeFontStyle, HeaderSize, Image, Language, Presentation,
	PresentationState, Slide, SlideNode,
};
use iced::*;
use iced_native::image::Data as ImageData;

type Element = iced::Element<'static, <App as Application>::Message>;

pub fn welcome_screen() -> Element {
	let welcome_msg_txt = Text::new("Presentation")
		.width(Length::Fill)
		.size(100)
		.color([1.0, 1.0, 1.0])
		.horizontal_alignment(HorizontalAlignment::Center)
		.vertical_alignment(VerticalAlignment::Center);

	Column::new()
		.spacing(20)
		.align_items(Align::Center)
		.push(welcome_msg_txt)
		.into()
}

static DEFAULT_SLIDE: Slide = Slide {
	nodes: Vec::new(),
	background: None,
};

pub fn presentation(presentation: &Presentation, state: &PresentationState) -> Element {
	let slide = match presentation.slides.get(state.slide_idx) {
		Some(v) => v,
		None => {
			log::error!(
				"Invalid slide idx: {}. Slide count: {}",
				state.slide_idx,
				presentation.slides.len()
			);
			&DEFAULT_SLIDE
		}
	};

	let mut column = Column::new().spacing(10).align_items(Align::Center);

	for element in &slide.nodes {
		match element {
			SlideNode::Header(size, txt) => {
				column = column.push(header(*size, txt));
			}
			SlideNode::Text(txt) => {
				column = column.push(text(txt));
			}
			SlideNode::UnnumberedList(list) => {
				column = column.push(unnumbered_list(list));
			}
			SlideNode::NumberedList(list) => {
				column = column.push(numbered_list(list));
			}
			SlideNode::Image(Image {
				path: _name,
				alt_text,
				params,
				handle,
			}) => {
				let image: iced::Element<_> = match handle {
					Some(ref handle) => {
						let mut scaled_height = None;
						match handle.data() {
							ImageData::Pixels { height, .. } => {
								if let Some(scale) = params.scale {
									scaled_height = Some(*height as f64 * (scale as f64 / 100.0));
									log::info!(
										"setting img height to {:?} from {}",
										scaled_height,
										*height
									);
								}
							}
							ImageData::Path(_) => {
								log::error!("image data contains a path variant");
							}
							ImageData::Bytes(_) => {
								log::error!("image data contains a bytes variant");
							}
						}
						let mut image = image::Image::new(handle.clone());

						if let Some(scaled_height) = scaled_height {
							log::info!("setting img height to {}", scaled_height);
							image = image.height(Length::Units(scaled_height as u16));
						}

						image.into()
					}
					None => text(alt_text).into(),
				};

				column = column.push(image);
			}
			SlideNode::CodeBlock(lang, params, txt) => {
				column = column.push(code_block(*lang, params, txt));
			}
			SlideNode::Comment(_) => continue,
		}
	}

	//TODO: Background image. Tracking issue: https://github.com/hecrj/iced/issues/450

	Container::new(column)
		.width(Length::Fill)
		.height(Length::Fill)
		.center_x()
		.center_y()
		.into()
}

static WHITE: [f32; 3] = [1.0, 1.0, 1.0];

fn header(size: HeaderSize, txt: &str) -> Element {
	Row::new()
		.padding(20)
		.push(
			Text::new(txt)
				.width(Length::Fill)
				// .height(Length::Fill)
				.size(size.to_font_size())
				.color(WHITE)
				.font(fonts::LATO_BOLD)
				.horizontal_alignment(HorizontalAlignment::Center)
				.vertical_alignment(VerticalAlignment::Center),
		)
		.into()
}

fn text(txt: &str) -> Element {
	Text::new(txt)
		.width(Length::Fill)
		.size(42)
		.color(WHITE)
		.font(fonts::LATO_REGULAR)
		.horizontal_alignment(HorizontalAlignment::Center)
		.vertical_alignment(VerticalAlignment::Center)
		.into()
}

const BULLET_CHAR: char = '\u{2022}';

fn unnumbered_list(list: &Vec<String>) -> Element {
	Column::with_children(
		list.iter()
			.map(|element| {
				Text::new(format!("\t{} {}", BULLET_CHAR, element))
					.width(Length::Shrink)
					.size(42)
					.color(WHITE)
					.font(fonts::LATO_REGULAR)
					.horizontal_alignment(HorizontalAlignment::Left)
					.vertical_alignment(VerticalAlignment::Center)
					.into()
			})
			.collect(),
	)
	.into()
}

fn numbered_list(list: &Vec<String>) -> Element {
	Column::with_children(
		list.iter()
			.enumerate()
			.map(|(idx, element)| {
				Text::new(format!("\t{}. {}", idx + 1, element))
					.width(Length::Shrink)
					.size(42)
					.color(WHITE)
					.font(fonts::LATO_REGULAR)
					.horizontal_alignment(HorizontalAlignment::Left)
					.vertical_alignment(VerticalAlignment::Center)
					.into()
			})
			.collect(),
	)
	.into()
}

// static SOLARIZED_GRAY: [f32; 3] = [131.0 / 255.0, 148.0 / 255.0, 150.0 / 255.0];

// static BLUE: [f32; 3] = [3.8 / 255.0, 94.9 / 255.0, 188.6 / 255.0];
// static RED: [f32; 3] = [193.3 / 255.0, 23.4 / 255.0, 88.5 / 255.0];

fn code_block(lang: Language, params: &CodeBlockParams, txt: &str) -> Element {
	use syntect::easy::HighlightLines;
	use syntect::highlighting::{Color, Style, ThemeSet};
	use syntect::parsing::SyntaxSet;

	let solarized_theme = ThemeSet::load_defaults().themes["Solarized (dark)"].to_owned();
	let syntax_set = SyntaxSet::load_defaults_newlines();

	let font_size = params.font_size.unwrap_or(38);
	let font = match params.font_style {
		Some(CodeFontStyle::Regular) => fonts::CASCADIA_CODE_REGULAR,
		Some(CodeFontStyle::Bold) => fonts::CASCADIA_CODE_BOLD,
		Some(CodeFontStyle::SemiBold) => fonts::CASCADIA_CODE_SEMI_BOLD,
		Some(CodeFontStyle::Light) => fonts::CASCADIA_CODE_LIGHT,
		Some(CodeFontStyle::SemiLight) => fonts::CASCADIA_CODE_SEMI_LIGHT,
		Some(CodeFontStyle::ExtraLight) => fonts::CASCADIA_CODE_EXTRA_LIGHT,
		None => fonts::CASCADIA_CODE_REGULAR,
	};

	let syntax_ref = match lang {
		Language::PlainText => Some(syntax_set.find_syntax_plain_text()),
		Language::Rust => syntax_set.find_syntax_by_name("Rust"),
	}
	.unwrap_or_else(|| syntax_set.find_syntax_plain_text());

	let mut highlighter = HighlightLines::new(syntax_ref, &solarized_theme);

	let rows: Vec<Element> = txt
		.replace("\t", "    ")
		.lines()
		.map(|mut line| {
			if line.is_empty() {
				line = " ";
			}

			let ranges: Vec<(Style, &str)> = highlighter.highlight(line, &syntax_set);

			Row::with_children(
				ranges
					.into_iter()
					.map(|(style, str)| {
						let Color { r, b, g, a } = style.foreground;
						Text::new(str)
							.width(Length::Shrink)
							.size(font_size)
							.color(iced::Color::from_rgba8(r, g, b, f32::from(a) / 255.0))
							.font(font)
							.into()
					})
					.collect(),
			)
			.into()
		})
		.collect();

	Container::new(Column::with_children(rows))
		.padding(10)
		.style(styles::CodeBlockContainer)
		.into()
}

static SOLARIZED_BASE03: [f32; 3] = [0.0 / 255.0, 43.0 / 255.0, 54.0 / 255.0];

mod styles {
	use crate::views::SOLARIZED_BASE03;
	use iced::container::{self, Style};
	use iced::Background;

	pub struct CodeBlockContainer;

	impl container::StyleSheet for CodeBlockContainer {
		fn style(&self) -> Style {
			container::Style {
				text_color: None,
				background: Some(Background::Color(SOLARIZED_BASE03.into())),
				border_radius: 10.0,
				border_width: 0.0,
				border_color: Default::default(),
			}
		}
	}
}

#[allow(dead_code)]
pub mod fonts {
	macro_rules! font {
        ($($name: ident : $filename: expr $(,)? ),*) => {
            $(
                pub static $name: iced::Font = iced::Font::External {
                    name: $filename,
                    bytes: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/", $filename)),
                };
            )*
        };
    }

	font![
		LATO_BLACK: "Lato-BlackItalic.ttf",
		LATO_BLACK_ITALIC: "Lato-BlackItalic.ttf",
		LATO_BOLD: "Lato-Bold.ttf",
		LATO_BOLD_ITALIC: "Lato-BoldItalic.ttf",
		LATO_ITALIC: "Lato-Italic.ttf",
		LATO_LIGHT: "Lato-Light.ttf",
		LATO_LIGHT_ITALIC: "Lato-LightItalic.ttf",
		LATO_REGULAR: "Lato-Regular.ttf",
		LATO_THIN: "Lato-Thin.ttf",
		LATO_THINITALIC: "Lato-ThinItalic.ttf",

		CASCADIA_CODE_BOLD: "CascadiaCode-Bold.ttf",
		CASCADIA_CODE_EXTRA_LIGHT: "CascadiaCode-ExtraLight.ttf",
		CASCADIA_CODE_LIGHT: "CascadiaCode-Light.ttf",
		CASCADIA_CODE_REGULAR: "CascadiaCode-Regular.ttf",
		CASCADIA_CODE_SEMI_BOLD: "CascadiaCode-SemiBold.ttf",
		CASCADIA_CODE_SEMI_LIGHT: "CascadiaCode-SemiLight.ttf",
	];
}
