use super::{segment, Segment};

/// Handle any valid URL tag.
///
/// This has several cases:
///  * [url]http://example.com/[/url] target=text
///  * [url="http://example.com/"]Foo[/url] quote-delimited target
///  * [url=example.com]Bar[/url] non-delimited target
named!(pub url(&str) -> Segment,
    alt_complete!(
        map!(
            delimited!(tag_no_case!("[url]"),
                       take_until_no_case!("[/url]"),
                       tag_no_case!("[/url]")),
            |body| Segment::Link {
                target: body,
                text: vec![Segment::Text(body)],
            }
        )
        | do_parse!(
            tag_no_case!("[url=\"") >>
            target: take_until_and_consume!("\"]") >>
            text: many0!(call!(segment, "[/url]")) >>
            tag_no_case!("[/url]") >>
            ( Segment::Link { target, text } )
        )
        | do_parse!(
            tag_no_case!("[url=") >>
            target: take_until_and_consume!("]") >>
            text: many0!(call!(segment, "[/url]")) >>
            tag_no_case!("[/url]") >>
            ( Segment::Link { target, text } )
        )
    )
);

#[test]
fn url_parses() {
    use super::DecorationStyle;

    assert_eq!(
        url("[URL]example.com[/URL]"),
        Ok((
            "",
            Segment::Link {
                target: "example.com",
                text: vec![Segment::Text("example.com")],
            }
        ))
    );
    assert_eq!(
        url("[url=example.com/\"quote\"]for [i]example[/url]"),
        Ok((
            "",
            Segment::Link {
                target: "example.com/\"quote\"",
                text: vec![Segment::Text("for [i]example")],
            }
        ))
    );
    assert_eq!(
        url("[url=\"example.com\"][b]orly?[/b][/url]more"),
        Ok((
            "more",
            Segment::Link {
                target: "example.com",
                text: vec![Segment::Decorated {
                    style: DecorationStyle::Bold,
                    text: vec![Segment::Text("orly?")],
                }],
            }
        ))
    );
}
