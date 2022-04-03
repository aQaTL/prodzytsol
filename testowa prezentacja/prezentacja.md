Link: [Rust Homepage](https://rust-lang.org)

---

### Bloczek kodu

```rust
fn main() {
    println!("Hello");
}
```

---

| font_size: 20
| font_style: semi_bold
```rust
fn parse_code_block_params(mut input: &str) -> IResult<&str, CodeBlockParams> {
    let mut params = CodeBlockParams::default();

    loop {
        let tail = match char::<_, nom::error::Error<&str>>('|')(input) {
            Ok((tail, _)) => tail,
            Err(_) => return Ok((input, params)),
        };
        let (tail, _) = space1(tail)?;

        let (tail, _) = alt((
            preceded(
                tuple((tag("font_size:"), space1)),
                map(map_res(digit1, str::parse::<u16>), |font_size| {
                    params.font_size = Some(font_size);
                }),
            ),
            preceded(
                tuple((tag("font_style:"), space1)),
                map(
                    alt((
                        map(tag("regular"), |_| CodeFontStyle::Regular),
                        map(tag("bold"), |_| CodeFontStyle::Bold),
                        map(tag("semi_bold"), |_| CodeFontStyle::SemiBold),
                        map(tag("light"), |_| CodeFontStyle::Light),
                        map(tag("semi_light"), |_| CodeFontStyle::SemiLight),
                        map(tag("extra_light"), |_| CodeFontStyle::ExtraLight),
                    )),
                    |font_style| {
                        params.font_style = Some(font_style);
                    },
                ),
            ),
        ))(tail)?;

        let (tail, _) = till_pat_consuming("\n").parse(tail)?;
        input = tail;
    }
}
```

---

# Hello

To jest moja prezentacja

~ Autor prezentacji

--- 

## Drugi slajd

- Lista
- nienumerowana

1. Lista
1. numerowana
1234. (numerki nie mają znaczenia)
1. (więc się nie pomylisz)

---![](assets/generic-background.jpg)

---

### Obrazki

![Ferris the crab](assets/ferris.png)

![Ferris the dancing crab](assets/dancing-ferris.gif)

(O nie, gify jeszcze nie działają)

---

### Ferris zmniejszony

![Ferris the crab](assets/ferris.png){ scale: 50%; }

---


