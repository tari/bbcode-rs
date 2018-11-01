#[macro_use]
extern crate nom;

mod quote;

/// Any logical segment of data- a tag or plain text.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Segment<'a> {
    PlainText(&'a str),
    Quote(quote::Quote<'a>),
}

named!(pub parse(&str) -> Vec<Segment>,
    many1!(segment)
);

/*named!(segment(&str) -> Segment,
    alt!(
        quote::quote => { |q| Segment::Quote(q) } |
        take_till1!(segment) => { |t: &str| Segment::PlainText(t) }
    )
);*/

named!(segment(&str) -> Segment,
    alt!(
        coded_segment | map!(text_segment, Segment::PlainText)
    )
);

named!(coded_segment(&str) -> Segment,
    alt!(
        map!(quote::quote, Segment::Quote)
    )
);

/// Matches any nonzero amount of input that is not a coded_segment.
fn text_segment(input: &str) -> nom::IResult<&str, &str, u32> {
    println!("text_segment {}", input);
    if input.len() == 0 {
        use nom::{Err::Incomplete, Needed::Unknown};
        return Err(Incomplete(Unknown));
    }

    // Text segment applies only if no coded segment was found, so we do not
    // consider the first character of input.
    for (idx, _) in input.char_indices().skip(1) {
        match segment(&input[idx..]) {
            // Got a hit, return everything up to it.
            Ok(_) => return Ok(input.split_at(idx)),
            // Not a valid coded segment for any reason, so consume.
            Err(_) => continue,
        }
    }

    // Fell off the end of input without finding a coded segment,
    // so return all of it.
    Ok((&input[input.len()..], input))
}
