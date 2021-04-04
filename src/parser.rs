use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::error::ParseError;
use nom::{FindSubstring, IResult, InputTake, Parser};

use crate::{HeaderSize, Presentation, Slide, SlideNode};
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

fn parse_text_section(input: &str) -> IResult<&str, String> {
	let (tail, text_str) = till_pat_consuming("\n\n").parse(input)?;

	Ok((tail, text_str.trim().to_string()))
}

fn parse_slide_node(input: &str) -> IResult<&str, SlideNode> {
	alt((
		map(parse_header, |(header_size, header)| {
			SlideNode::Header(header_size, header)
		}),
		map(parse_text_section, |text| SlideNode::Text(text)),
	))(input)
}

fn parse_slide(mut input: &str) -> IResult<&str, Slide> {
	let mut slide_nodes = Vec::new();

	while !input.is_empty() {
		let (tail, slide_node) = parse_slide_node(input)?;
		input = tail;
		match slide_node {
			SlideNode::Text(t) if t == "---" => break,
			_ => slide_nodes.push(slide_node),
		}
	}

	Ok((input, Slide(slide_nodes)))
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

	#[test]
	fn parse_presentation_test() -> Result<()> {
		let presentation =
			parse_presentation(String::from("test presentation"), SAMPLE_PRESENTATION)?;
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
		let expected = Slide(vec![
			SlideNode::Header(HeaderSize::Three, String::from("hi1")),
			SlideNode::Header(HeaderSize::Two, String::from("Hello 2")),
		]);

		let (_, slide_nodes) = parse_slide("### hi1\n\n## Hello 2\n\n")?;
		assert_eq!(slide_nodes, expected);

		let (_, slide_nodes) = parse_slide("### hi1\n\n## Hello 2")?;
		assert_eq!(slide_nodes, expected);

		let (_, slide_nodes) = parse_slide("### hi1\n\n## Hello 2\n")?;
		assert_eq!(slide_nodes, expected);

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
