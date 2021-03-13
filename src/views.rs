use crate::{App, HeaderSize, Presentation, PresentationState, Slide, SlideNode};
use iced::*;
use log::debug;

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

static DEFAULT_SLIDE: Slide = default_slide();

const fn default_slide() -> Slide {
    Slide(Vec::new())
}

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
            SlideNode::NumberedList(_) => {}
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
    debug!("font size {}", size.to_font_size());
    Text::new(txt)
        .width(Length::Fill)
        // .height(Length::Fill)
        .size(size.to_font_size())
        .color(WHITE)
        // .font(Font::External {
        //     name: "Lato-Bold",
        //     bytes: fonts::LATO_BOLD,
        // })
        .font(fonts::LATO_BOLD)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}

fn text(txt: &str) -> Element {
    Text::new(txt)
        .width(Length::Fill)
        .size(42)
        .color(WHITE)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}

#[allow(dead_code)]
pub mod fonts {
    const fn new_external_font(name: &'static str, bytes: &'static [u8]) -> iced::Font {
        iced::Font::External { name, bytes }
    }
    macro_rules! font {
        ($($name: ident : $filename: expr $(,)? ),*) => {
            $(
                pub static $name: iced::Font = new_external_font(
                    $filename,
                    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/", $filename)),
                );
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
    ];
}
