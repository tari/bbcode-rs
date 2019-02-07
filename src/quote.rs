//! Block quotes.

use super::Segment::Quote;
use super::{segment, Segment};

named!(pub quote(&str) -> Segment,
    map!(
        terminated!(
            pair!(qhead, many0!(call!(segment, "[/quote]"))),
            tag_no_case!("[/quote]")
        ),
        |(attribution, body)| Quote { attribution, body }
    )
);

named!(qhead(&str) -> Option<&str>,
    delimited!(
        tag_no_case!("[quote"),
        opt!(preceded!(tag!("=\""), take_until_and_consume!("\""))),
        char!(']')
    )
);

#[test]
fn just_qhead() {
    assert_eq!(qhead("[quote]"), Ok(("", None)));
    assert_eq!(qhead("[quote=\"たみや\"]"), Ok(("", Some("たみや"))));
}

#[cfg(test)]
mod tests {
    use super::super::Segment;
    use super::quote as real_quote;
    use super::Quote;

    named!(quote(&str) -> Segment, dbg!(real_quote));

    #[test]
    fn empty_quote() {
        assert_eq!(
            quote("[quote][/quote])"),
            Ok((
                ")",
                Quote {
                    attribution: None,
                    body: vec![],
                }
            ))
        );
    }

    #[test]
    fn quote_without_attribution() {
        assert_eq!(
            quote("[quote]lol[/quote]More stuff"),
            Ok((
                "More stuff",
                Quote {
                    attribution: None,
                    body: vec![Segment::Text("lol")]
                }
            ))
        );
    }
}
