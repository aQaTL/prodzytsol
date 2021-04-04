use anyhow::Result;
use crate::{Presentation, HeaderSize, Slide, SlideNode};
use nom::{IResult, Parser};
use nom::sequence::delimited;
use nom::character::complete::{char, line_ending, space1};
use nom::bytes::complete::{is_not, take_while_m_n, take_until};
use nom::error::FromExternalError;
use nom::complete::tag;
use nom::branch::alt;
use nom::combinator::{map, iterator, all_consuming};

static SAMPLE_PRESENTATION: &str = r######"## Wprowadzenie do Rusta, dla tych, którzy już trochę programować umieją

Maciej Sołtys

---

# Rust


tutaj jakiś wstęp / najważniejsze punkty

"######;

fn parse_header(input: &str) -> IResult<&str, (HeaderSize, String)> {
    let (tail, header) = take_while_m_n(1, 5, |c| c == '#')(input)?;
    // SAFETY: take_while_m_n line above has range of 1..=5
    let header_size = unsafe { HeaderSize::from_u8_unchecked(header.len() as u8) };

    let (tail, _) = space1(tail)?;

    let (tail, header_str) = take_until("\n\n")(tail)?;

    Ok((&tail[2..], (header_size, header_str.to_string())))
}

fn parse_text_section(input: &str) -> IResult<&str, String> {
    let (tail, text_str) = take_until("\n\n")(input)?;

    Ok((&tail[2..], text_str.to_string()))
}

fn parse_slide_node(input: &str) -> IResult<&str, SlideNode> {
    alt((
        map(parse_header, |(header_size, header)| SlideNode::Header(header_size, header)),
        map(parse_text_section, |text| SlideNode::Text(text)),
    ))(input)
}

fn parse_slide(input: &str) -> IResult<&str, Slide> {
    println!("parsing slide: \"{}\"", input);
    let mut it = iterator(input, parse_slide_node);
    let slide_nodes = it.take_while(|slide_node| {
          !matches!(slide_node, SlideNode::Text(t) if t == "---" )
    }).collect::<Vec<_>>();
    let (tail, _) = it.finish()?;
    Ok((tail, Slide(slide_nodes)))
}

pub fn parse_slides(input: &str) -> IResult<&str, Vec<Slide>> {
    println!("parsing slides");
    let mut it = iterator(input, parse_slide);
    let slides = it.collect::<Vec<_>>();
    let (tail, _) = it.finish()?;
    Ok((tail, slides))
}

pub fn parse_presentation(title: String, input: &str) -> Result<Presentation> {
    let (tail, slides) = match parse_slides(&input) {
      Ok(v) =>v,
        Err(e) => anyhow::bail!("parse_presentation failed with: {:?}", e),
    };
    println!("tail: {}", tail);
    Ok(Presentation {
        title,
        slides,
    })
}

#[test]
fn parse_presentation_test() -> Result<()> {
    let presentation = parse_presentation(String::from("test presentation"), SAMPLE_PRESENTATION)?;
    println!("{:#?}", presentation);

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
    let input = "##### Hello";
    let (tail, header) = parse_header(SAMPLE_PRESENTATION)?;
    println!("{:?}", header);
    println!("{:?}", tail);
    let (tail, text) = parse_text_section(tail)?;
    println!("{:?}", text);
    println!("{:?}", tail);
    // println!("{:?}", parse_text_section(SAMPLE_PRESENTATION));
    Ok(())
}