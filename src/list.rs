//! Lists of items.

use super::Segment::List;
use super::{segment, Segment};

/// The general appearance of a list.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum ListStyle {
    /// No particular order; items usually marked with bullet points.
    ///
    /// Like CSS `list-style-type` `disc`.
    Unordered,
    /// Items numbered in increasing order.
    ///
    /// Like CSS `list-style-type` `decimal`.
    Numeric,
    /// Items marked with latin letters.
    ///
    /// Like CSS `list-style-type` `lower-alpha`.
    Alphabetic,
}

/// Recognizes a list: `[list][*] Item [*] Item[/list]`
named!(pub list(&str) -> Segment,
    do_parse!(
        style: listhead >>
        items: many0!(preceded!(tag_no_case!("[*]"),
                                many0!(call!(segment, &["[*]", "[/list]"])))) >>
        tag_no_case!("[/list]") >>
        (List { style, items })
    )
);

named!(listhead(&str) -> ListStyle,
    map!(
        delimited!(
            tag_no_case!("[list"),
            opt!(
                preceded!(char!('='),
                          alt!(char!('a')
                               | char!('1'))
                )
            ),
            char!(']')
        ),
        |style| match style {
            None => ListStyle::Unordered,
            Some('1') => ListStyle::Numeric,
            Some('a') => ListStyle::Alphabetic,
            Some(_) => unreachable!(),
        }
    )
);

#[test]
fn list_with_multiple_items() {
    let (tail, x) = list("[list][*] One\n[*] Two[/list]Tail").expect("Should parse successfully");
    assert_eq!(
        x,
        List {
            style: ListStyle::Unordered,
            items: vec![vec![Segment::Text(" One\n")], vec![Segment::Text(" Two")],]
        }
    );
    assert_eq!(tail, "Tail");
}

#[test]
fn empty_list() {
    assert_eq!(
        list("[list=a][/list]"),
        Ok((
            "",
            List {
                style: ListStyle::Alphabetic,
                items: vec![],
            }
        ))
    );
}
