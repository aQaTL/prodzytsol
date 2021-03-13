use crate::{App, Presentation, PresentationState, Slide};
use iced::*;

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
    Slide { header: None }
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

    let welcome_msg_txt = header(slide);

    let column = Column::new()
        .spacing(20)
        .height(Length::Fill)
        .align_items(Align::Center)
        .push(welcome_msg_txt);

    column.into()
}

fn header(slide: &Slide) -> Element {
    let header = match slide.header.as_deref() {
        Some(v) => v,
        None => return Text::new("").into(),
    };

    Text::new(format!("{}", header))
        .width(Length::Fill)
        .size(100)
        .color([1.0, 1.0, 1.0])
        .horizontal_alignment(HorizontalAlignment::Center)
        .into()
}
