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

    let mut column = Column::new()
        // .height(Length::Fill)
        // .width(Length::Fill)
        .align_items(Align::Center);

    for element in &slide.0 {
        match element {
            SlideNode::Header(size, txt) => {
                column = column.push(header(*size, txt));
            }
            SlideNode::Text(_) => {}
            SlideNode::NumberedList(_) => {}
        }
    }

    Row::new()
        // .height(Length::Fill)
        // .width(Length::Fill)
        .align_items(Align::Center)
        .push(column)
        .into()
    // column.into()
}

fn header(size: HeaderSize, txt: &str) -> Element {
    debug!("font size {}", size.to_font_size());
    Text::new(format!("{}", txt))
        .width(Length::Fill)
        // .height(Length::Fill)
        .size(size.to_font_size())
        .color([1.0, 1.0, 1.0])
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .into()
}
