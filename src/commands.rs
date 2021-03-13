use anyhow::Result;

use crate::{HeaderSize, Presentation, Slide, SlideNode};

pub type LoadFromArgsResult = Result<Presentation>;

pub async fn load_from_args() -> LoadFromArgsResult {
    let presentation = Presentation {
        title: "Rust - wprowadzenie.aqaprez".to_string(),
        slides: vec![
            Slide(vec![
                SlideNode::Header(
                    HeaderSize::Two,
                    String::from(
                        "Wprowadzenie do Rusta dla tych, którzy już trochę programować umieją",
                    ),
                ),
                SlideNode::Text(String::from("Maciej Sołtys")),
                // SlideNode::Header(HeaderSize::Four, String::from("Maciej Sołtys")),
            ]),
            Slide(vec![
                SlideNode::Header(
                    HeaderSize::Three,
                    String::from("Wersja dla tych, którzy umieją, czyli"),
                ),
                SlideNode::UnnumberedList(vec![
                    String::from("Szybki przegląd składni, typów"),
                    String::from("Feature'y"),
                    String::from("Różnice (C++ / Java / C# / Go)"),
                ]),
            ]),
        ],
    };
    Ok(presentation)
}
