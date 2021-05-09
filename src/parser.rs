use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while_m_n};
use nom::character::complete::{char, digit1, space1};
use nom::combinator::{map, map_res, opt};
use nom::error::ParseError;
use nom::sequence::{delimited, preceded, tuple};
use nom::{FindSubstring, IResult, InputTake, Parser};

use crate::{HeaderSize, Image, Language, Presentation, Slide, SlideNode};
use nom::multi::many1;
use std::path::PathBuf;

#[cfg(test)]
static SAMPLE_PRESENTATION: &str = r######"## Wprowadzenie do Rusta, dla tych, którzy już trochę programować umieją

Maciej Sołtys

---

# Rust


tutaj jakiś wstęp / najważniejsze punkty

hahalol

---

### Another one

- my
- super awesome
- list of things :)
"######;

fn till_pat_consuming<'i: 'substr, 'substr, E: ParseError<&'i str>>(
	substr: &'substr str,
) -> impl Parser<&'i str, &'i str, E> + 'substr {
	move |input: &'i str| match input.find_substring(substr) {
		Some(index) => {
			let (tail, value) = input.take_split(index);
			Ok((&tail[substr.len()..], value))
		}
		None => Ok((&input[0..0], input)),
	}
}

fn parse_header(input: &str) -> IResult<&str, (HeaderSize, String)> {
	let (tail, header) = take_while_m_n(1, 5, |c| c == '#')(input)?;
	// SAFETY: take_while_m_n line above has range of 1..=5
	let header_size = unsafe { HeaderSize::from_u8_unchecked(header.len() as u8) };

	let (tail, _) = space1(tail)?;

	let (tail, header_str) = till_pat_consuming("\n\n").parse(tail)?;

	Ok((tail, (header_size, header_str.trim().to_string())))
}

fn parse_unnumbered_list(input: &str) -> IResult<&str, Vec<String>> {
	let (tail, items) = map(
		many1(preceded(
			tuple((char('-'), space1)),
			till_pat_consuming("\n"),
		)),
		|items| {
			items
				.into_iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>()
		},
	)(input)?;
	let (tail, _) = tag("\n")(tail)?;

	Ok((tail, items))
}

fn parse_numbered_list(input: &str) -> IResult<&str, Vec<String>> {
	let (tail, items) = map(
		many1(preceded(
			tuple((digit1, opt(char('.')), space1)),
			till_pat_consuming("\n"),
		)),
		|items| {
			items
				.into_iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>()
		},
	)(input)?;
	let (tail, _) = tag("\n")(tail)?;

	Ok((tail, items))
}

fn parse_code_block(input: &str) -> IResult<&str, (Language, String)> {
	let (tail, _) = tag("```")(input)?;
	let (tail, language) =
		map_res(till_pat_consuming("\n"), |lang| lang.parse::<Language>())(tail)?;
	let (tail, code_block) = till_pat_consuming("```").parse(tail)?;

	let (tail, _) = till_pat_consuming("\n\n").parse(tail)?;

	Ok((tail, (language, code_block.to_string())))
}

fn parse_image(input: &str) -> IResult<&str, Image> {
	let (tail, _) = char('!')(input)?;
	let (tail, alt_text) = delimited(char('['), opt(is_not("]")), char(']'))(tail)?;
	let (tail, path) = delimited(char('('), is_not(")"), char(')'))(tail)?;

	let (tail, _) = till_pat_consuming("\n\n").parse(tail)?;

	Ok((
		tail,
		Image {
			path: path.to_string(),
			alt_text: alt_text.map(ToString::to_string).unwrap_or_default(),
			handle: None,
		},
	))
}

fn parse_text_section(input: &str) -> IResult<&str, String> {
	let (tail, text_str) = till_pat_consuming("\n\n").parse(input)?;

	Ok((tail, text_str.trim().to_string()))
}

fn parse_slide_node(input: &str) -> IResult<&str, SlideNode> {
	alt((
		map(parse_header, |(header_size, header)| {
			SlideNode::Header(header_size, header)
		}),
		map(parse_unnumbered_list, |items| {
			SlideNode::UnnumberedList(items)
		}),
		map(parse_numbered_list, |items| SlideNode::NumberedList(items)),
		map(parse_code_block, |(language, code_block)| {
			SlideNode::CodeBlock(language, code_block)
		}),
		map(parse_image, |image| SlideNode::Image(image)),
		map(parse_text_section, |text| SlideNode::Text(text)),
	))(input)
}

fn parse_slide_divider(input: &str) -> IResult<&str, Option<Image>> {
	let (tail, _) = tag("---")(input)?;
	let (tail, background) = opt(parse_image)(tail)?;
	Ok((tail, background))
}

fn parse_slide(mut input: &str) -> IResult<&str, Slide> {
	let mut slide_nodes = Vec::new();

	let mut background = None;

	while !input.is_empty() {
		if let Ok((tail, new_background)) = parse_slide_divider(input) {
			if slide_nodes.is_empty() {
				background = new_background;
			} else {
				return Ok((
					input,
					Slide {
						nodes: slide_nodes,
						background,
					},
				));
			}
			input = tail;
			continue;
		}
		let (tail, slide_node) = parse_slide_node(input)?;
		input = tail;
		slide_nodes.push(slide_node);
	}

	Ok((
		input,
		Slide {
			nodes: slide_nodes,
			background,
		},
	))
}

pub fn parse_slides(mut input: &str) -> IResult<&str, Vec<Slide>> {
	let mut slides = Vec::new();

	while !input.is_empty() {
		let (tail, slide) = parse_slide(input)?;
		slides.push(slide);
		input = tail;
	}

	Ok((input, slides))
}

pub fn parse_presentation(title: String, path: PathBuf, input: &str) -> Result<Presentation> {
	let (_, slides) = match parse_slides(&input) {
		Ok(v) => v,
		Err(e) => anyhow::bail!("parse_presentation failed with: {:?}", e),
	};
	Ok(Presentation {
		title,
		path,
		slides,
	})
}

#[cfg(test)]
mod tests {
	use anyhow::Result;

	use super::*;
	use crate::{Image, Language};

	#[test]
	fn parse_presentation_test() -> Result<()> {
		let presentation = parse_presentation(
			String::from("test presentation"),
			PathBuf::from("test presentation.md"),
			SAMPLE_PRESENTATION,
		)?;
		println!("{:#?}", presentation);

		Ok(())
	}

	#[test]
	fn parse_header_test() -> Result<()> {
		let (_, (header_size, header)) = parse_header("# hello")?;
		assert!(matches!(header_size, HeaderSize::One));
		assert_eq!(&header, "hello");
		Ok(())
	}

	#[test]
	fn parse_header_with_newlines() -> Result<()> {
		let (_, (header_size, header)) = parse_header("##### hi\n\n")?;
		assert!(matches!(header_size, HeaderSize::Five));
		assert_eq!(&header, "hi");
		Ok(())
	}

	#[test]
	fn parse_headers() -> Result<()> {
		let expected = Slide {
			nodes: vec![
				SlideNode::Header(HeaderSize::Three, String::from("hi1")),
				SlideNode::Header(HeaderSize::Two, String::from("Hello 2")),
			],
			..Default::default()
		};

		let (_, slide_nodes) = parse_slide("### hi1\n\n## Hello 2\n\n")?;
		assert_eq!(slide_nodes, expected);

		let (_, slide_nodes) = parse_slide("### hi1\n\n## Hello 2")?;
		assert_eq!(slide_nodes, expected);

		let (_, slide_nodes) = parse_slide("### hi1\n\n## Hello 2\n")?;
		assert_eq!(slide_nodes, expected);

		Ok(())
	}

	#[test]
	fn parse_image() -> Result<()> {
		let expected = Image {
			path: "ferris.png".to_string(),
			alt_text: "Ferris the crab".to_string(),
			// handle isn't compared
			handle: None,
		};

		let (_, image) = super::parse_image("![Ferris the crab](ferris.png)")?;
		println!("{:?}", image);
		println!("{:?}", expected);

		assert_eq!(image, expected);
		Ok(())
	}

	#[test]
	fn parse_image_with_empty_alt_text() -> Result<()> {
		let expected = Image {
			path: "ferris.png".to_string(),
			alt_text: "".to_string(),
			// handle isn't compared
			handle: None,
		};

		let (_, image) = super::parse_image("![](ferris.png)")?;
		assert_eq!(expected, image);

		Ok(())
	}

	#[test]
	fn parse_code_block() -> Result<()> {
		let expected = (
			Language::Rust,
			r#"enum Result<T, E> {
	Ok(T),
	Err(E),
}

fn main() {
	println!("Hello, World!");
}
"#
			.to_string(),
		);

		let (_, code_block) = super::parse_code_block(
			r#"```rust
enum Result<T, E> {
	Ok(T),
	Err(E),
}

fn main() {
	println!("Hello, World!");
}
```"#,
		)?;

		assert_eq!(expected, code_block);
		Ok(())
	}

	#[test]
	fn parse_unnumbered_list_slide() -> Result<()> {
		let expected = Slide {
			nodes: vec![SlideNode::UnnumberedList(vec![
				"Ala".to_string(),
				"ma".to_string(),
				"kota".to_string(),
			])],
			..Default::default()
		};

		let (_, slide) = super::parse_slide(
			r#"-    Ala
- ma
- kota

"#,
		)?;

		assert_eq!(slide, expected);
		Ok(())
	}

	#[test]
	fn parse_numbered_list_slide() -> Result<()> {
		let expected = Slide {
			nodes: vec![SlideNode::NumberedList(vec![
				"Ala".to_string(),
				"ma".to_string(),
				"kota".to_string(),
			])],
			background: None,
		};

		let (_, slide) = super::parse_slide(
			r#"1. Ala
2. ma
3.    kota

"#,
		)?;

		assert_eq!(slide, expected);
		Ok(())
	}

	#[test]
	fn parse_code_block_slide() -> Result<()> {
		let expected = Slide {
			nodes: vec![SlideNode::CodeBlock(
				Language::Rust,
				r#"enum Result<T, E> {
	Ok(T),
	Err(E),
}

fn main() {
	println!("Hello, World!");
}
"#
				.to_string(),
			)],
			..Default::default()
		};

		let (_, slide) = super::parse_slide(
			r#"```rust
enum Result<T, E> {
	Ok(T),
	Err(E),
}

fn main() {
	println!("Hello, World!");
}
```

"#,
		)?;

		assert_eq!(expected, slide);
		Ok(())
	}

	#[test]
	fn parse_image_slide() -> Result<()> {
		let expected = Slide {
			nodes: vec![SlideNode::Image(Image {
				path: "ferris.png".to_string(),
				alt_text: "ferris".to_string(),
				handle: None,
			})],
			..Default::default()
		};

		let (_, slide_nodes) = parse_slide("![ferris](ferris.png)\n\n")?;
		assert_eq!(slide_nodes, expected);

		Ok(())
	}

	#[test]
	fn parse_slide_background() -> Result<()> {
		let expected = Slide {
			nodes: vec![SlideNode::Text("Hello, World!".to_string())],
			background: Some(Image {
				path: "assets/generic-background.jpg".to_string(),
				alt_text: "".to_string(),
				handle: None,
			}),
		};

		let (_, slide) = parse_slide(
			r#"---![](assets/generic-background.jpg)

Hello, World!

"#,
		)?;

		assert_eq!(expected, slide);
		Ok(())
	}

	#[test]
	fn parse_slide_background_for_second_slide() -> Result<()> {
		let expected = vec![
			Slide {
				nodes: vec![SlideNode::Header(
					HeaderSize::One,
					"first slide".to_string(),
				)],
				background: None,
			},
			Slide {
				nodes: vec![SlideNode::Text("Hello, World!".to_string())],
				background: Some(Image {
					path: "assets/generic-background.jpg".to_string(),
					alt_text: "".to_string(),
					handle: None,
				}),
			},
		];

		let (_, slides) = parse_slides(
			r#"# first slide

---![](assets/generic-background.jpg)

Hello, World!

"#,
		)?;

		assert_eq!(expected, slides);
		Ok(())
	}

	#[test]
	fn parse_slide_test() -> anyhow::Result<()> {
		let (tail, slide) = parse_slide("hello\n\nworld")?;
		println!("{:#?}", slide);
		println!("{:?}", tail);
		Ok(())
	}

	#[test]
	fn parse_slide_node_test() -> anyhow::Result<()> {
		let (tail, slide_node) = parse_slide_node(SAMPLE_PRESENTATION)?;
		println!("{:?}", slide_node);
		println!("{:?}", tail);

		Ok(())
	}

	#[test]
	fn f() -> anyhow::Result<()> {
		let (tail, header) = parse_header(SAMPLE_PRESENTATION)?;
		println!("{:?}", header);
		println!("{:?}", tail);
		let (tail, text) = parse_text_section(tail)?;
		println!("{:?}", text);
		println!("{:?}", tail);
		// println!("{:?}", parse_text_section(SAMPLE_PRESENTATION));
		Ok(())
	}
}
