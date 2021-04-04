use crate::{App, HeaderSize, Image, Language, Presentation, PresentationState, Slide, SlideNode};
use iced::*;
use unicode_segmentation::UnicodeSegmentation;

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

static DEFAULT_SLIDE: Slide = Slide(Vec::new());

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

	for element in &slide.0 {
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
				name: _name,
				handle,
			}) => {
				column = column.push(image::Image::new(handle.clone()));
			}
			SlideNode::CodeBlock(lang, txt) => {
				column = column.push(code_block(*lang, txt));
			}
		}
	}

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

static SOLARIZED_GRAY: [f32; 3] = [131.0 / 255.0, 148.0 / 255.0, 150.0 / 255.0];

// static BLUE: [f32; 3] = [3.8 / 255.0, 94.9 / 255.0, 188.6 / 255.0];
// static RED: [f32; 3] = [193.3 / 255.0, 23.4 / 255.0, 88.5 / 255.0];

fn code_block(_lang: Language, txt: &str) -> Element {
	let rows: Vec<Element> = txt
		.replace("\t", "    ")
		.lines()
		.map(|mut line| {
			if line.is_empty() {
				line = " ";
			}

			Row::with_children(
				line.graphemes(true)
					.map(|grapheme| {
						Text::new(grapheme)
							.width(Length::Shrink)
							.size(38)
							.color(SOLARIZED_GRAY)
							.font(fonts::CASCADIA_CODE_REGULAR)
							.into()
					})
					.collect(),
			)
			.into()
		})
		.collect();

	Column::with_children(rows).into()
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
