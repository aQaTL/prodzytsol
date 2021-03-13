use anyhow::Result;

use crate::{Presentation, Slide};

pub type LoadFromArgsResult = Result<Presentation>;

pub async fn load_from_args() -> LoadFromArgsResult {
    let presentation = Presentation {
        title: "Rust - wprowadzenie.aqaprez".to_string(),
        slides: vec![Slide {
            header: Some(
                "Wprowadzenie do Rusta dla tych, którzy już trochę programować umieją".to_string(),
            ),
        }],
    };
    Ok(presentation)
}
