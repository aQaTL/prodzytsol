use anyhow::{anyhow, Context, Result};
use iced::image;
use std::path::Path;

use crate::{HeaderSize, Language, Presentation, Slide, SlideNode};

pub type LoadFromArgsResult = Result<Presentation>;

pub async fn load_from_args() -> LoadFromArgsResult {
	let file_path = std::env::args().skip(1).next();
	match file_path {
		Some(path) => load_from_file(&path).await,
		None => load_example().await,
	}
}

async fn load_from_file(path: &str) -> LoadFromArgsResult {
	let path = Path::new(path);

	let file = async_fs::read_to_string(path).await?;

	std::env::set_current_dir(
		path.parent()
			.ok_or(anyhow!("Can't get parent dir of {}", path.display()))?,
	)
	.context("Failed to set current_dir")?;

	let title = path.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string());

    crate::parser::parse_presentation(title, &file)
}

async fn load_example() -> LoadFromArgsResult {
	let slides = vec![
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
		Slide(vec![
			SlideNode::Header(HeaderSize::One, String::from("Ferris")),
			SlideNode::Image(load_image("ferris.png").await?),
		]),
		Slide(vec![
			SlideNode::Header(HeaderSize::Two, String::from("while loop")),
			SlideNode::CodeBlock(
				Language::Rust,
				String::from(
					r#"let mut a = 0;

while a < 10 {
	a += 1;
}"#,
				),
			),
		]),
		Slide(vec![
			SlideNode::Header(HeaderSize::Two, String::from("enum")),
			SlideNode::CodeBlock(
				Language::Rust,
				String::from(
					r#"enum SqrtResult {
	Success(f64),
	Fail(SqrtError),
}

enum SqrtError {
	NegativeNumber,
}

fn sqrt(n: f64) -> SqrtResult {
	if n < 0.0 {
		return SqrtResult::Fail(SqrtError::NegativeNumber);
	}

	let sqrt_result = n.sqrt();

	Ok(sqrt_result)
}"#,
				),
			),
		]),
	];

	let presentation = Presentation {
		title: "Rust - wprowadzenie.aqaprez".to_string(),
		slides,
	};
	Ok(presentation)
}

async fn load_image(path: &str) -> Result<image::Handle> {
	let handle = image::Handle::from_path(format!("assets/{}", path));
	Ok(handle)
}
