use super::{segment, Segment};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Quote<'a> {
    pub attribution: Option<&'a str>,
    pub body: Vec<Segment<'a>>
}

named!(pub quote(&str) -> Quote,
    map!(
        terminated!(
            pair!(qhead, many0!(segment)),
            tag!("[/quote]")
        ),
        |(attribution, body)| Quote { attribution, body }
    )
);

named!(qhead(&str) -> Option<&str>,
    delimited!(
        tag!("[quote"),
        opt!(preceded!(tag!("=\""), take_until_and_consume!("\""))),
        tag!("]")
    )
);

#[test]
fn just_qhead() {
    assert_eq!(qhead("[quote]"), Ok(("", None)));
    assert_eq!(qhead("[quote=\"たみや\"]"), Ok(("", Some("たみや"))));
}

#[cfg(test)]
mod tests {
    use super::Quote;
    use super::quote as real_quote;
    use super::super::Segment;

    named!(quote(&str) -> Quote, dbg_dmp!(real_quote));

    #[test]
    fn quote_without_attribution() {
        assert_eq!(quote("[quote]lol[/quote]More stuff"),
            Ok(("More stuff", Quote {
                attribution: None,
                body: vec![Segment::PlainText("lol")]
            })));
    }
}
