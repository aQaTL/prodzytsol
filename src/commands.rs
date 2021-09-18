use anyhow::{anyhow, Context, Result};
use iced::image;
use log::error;
use std::path::{Path, PathBuf};

use crate::{FileWatch, HeaderSize, Image, Language, Presentation, Slide, SlideNode};

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

	let title = path
		.file_name()
		.map(|x| x.to_string_lossy().to_string())
		.unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string());

	let presentation_dir = path
		.canonicalize()
		.with_context(|| format!("failed to canonicalize {}", path.display()))?;
	let presentation_dir = presentation_dir
		.parent()
		.ok_or(anyhow!("failed to get parent of {}", path.display()))?;

	let mut presentation =
		crate::parser::parse_presentation(title, presentation_dir.to_owned(), &file)?;

	presentation
		.slides
		.iter_mut()
		.flat_map(|slide| slide.nodes.iter_mut())
		.filter_map(|node| match node {
			SlideNode::Image(ref mut img) => Some(img),
			_ => None,
		})
		.for_each(|image| {
			let path = presentation_dir.join(&image.path);
			if !path.exists() || !path.is_file() {
				log::error!("{} not found", path.display());
				return;
			}

			let handle = image::Handle::from_path(path);
			image.handle = Some(handle);
		});

	Ok(presentation)
}

pub type StartFileWatcherResult = Option<FileWatch>;

pub async fn start_file_watcher(path: PathBuf) -> StartFileWatcherResult {
	match FileWatch::new(path).await {
		Ok(v) => Some(v),
		Err(e) => {
			error!("Failed to crate file watcher: {:?}", e);
			None
		}
	}
}

async fn load_example() -> LoadFromArgsResult {
	let slides = vec![
		Slide {
			nodes: vec![
				SlideNode::Header(
					HeaderSize::Two,
					String::from(
						"Wprowadzenie do Rusta dla tych, którzy już trochę programować umieją",
					),
				),
				SlideNode::Text(String::from("Maciej Sołtys")),
				// SlideNode::Header(HeaderSize::Four, String::from("Maciej Sołtys")),
			],
			..Default::default()
		},
		Slide {
			nodes: vec![
				SlideNode::Header(
					HeaderSize::Three,
					String::from("Wersja dla tych, którzy umieją, czyli"),
				),
				SlideNode::UnnumberedList(vec![
					String::from("Szybki przegląd składni, typów"),
					String::from("Feature'y"),
					String::from("Różnice (C++ / Java / C# / Go)"),
				]),
			],
			..Default::default()
		},
		Slide {
			nodes: vec![
				SlideNode::Header(HeaderSize::One, String::from("Ferris")),
				SlideNode::Image(load_image("ferris.png").await?),
			],
			..Default::default()
		},
		Slide {
			nodes: vec![
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
			],
			..Default::default()
		},
		Slide {
			nodes: vec![
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
			],
			..Default::default()
		},
	];

	let presentation = Presentation {
		title: "Example presentation".to_string(),
		path: PathBuf::from("."),
		slides,
	};
	Ok(presentation)
}

async fn load_image(path: &str) -> Result<Image> {
	let handle = Some(image::Handle::from_path(format!("assets/{}", path)));
	Ok(Image {
		path: path.to_string(),
		alt_text: "Ferris the crab".to_string(),
		handle,
	})
}
