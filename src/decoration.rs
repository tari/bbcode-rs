//! Plain text spans with additional decoration.

use super::{segment, Segment};
use palette::Srgb;

/// Styles that can be applied to decorated spans.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum DecorationStyle {
    Bold,
    Italic,
    Underline,
    /// Horizontally centered.
    Center,
    /// Colored with specified sRGB components (as in CSS).
    Color(u8, u8, u8),
}

fn styled(s: DecorationStyle) -> impl for<'a> Fn(Vec<Segment<'a>>) -> Segment<'a> {
    move |text| Segment::Decorated { style: s, text }
}

named!(pub decorated(&str) -> Segment,
    alt!(map!(bold, styled(DecorationStyle::Bold))
        | map!(italic, styled(DecorationStyle::Italic))
        | map!(underline, styled(DecorationStyle::Underline))
        | map!(center, styled(DecorationStyle::Center))
        | color
    )
);

/// Recognizes `[b]This text is bold[/b]`.
named!(pub bold(&str) -> Vec<Segment>,
    delimited!(
        tag_no_case!("[b]"),
        many0!(call!(segment, "[/b]")),
        tag_no_case!("[/b]")
    )
);

#[test]
fn bold_text() {
    assert_eq!(bold("[b]BOLD![/b]"), Ok(("", vec![Segment::Text("BOLD!")])));
}

named!(pub italic(&str) -> Vec<Segment>,
    delimited!(
        tag_no_case!("[i]"),
        many0!(call!(segment, "[/i]")),
        tag_no_case!("[/i]")
    )
);

named!(pub underline(&str) -> Vec<Segment>,
    delimited!(
        tag_no_case!("[u]"),
        many0!(call!(segment, "[/u]")),
        tag_no_case!("[/u]")
    )
);

#[test]
fn underlined_text() {
    assert_eq!(underline("[u]um[/u]"), Ok(("", vec![Segment::Text("um")])));
}

named!(pub center(&str) -> Vec<Segment>,
    delimited!(
        tag_no_case!("[center]"),
        many0!(call!(segment, "[/center]")),
        tag_no_case!("[/center]")
    )
);

named!(pub color(&str) -> Segment,
    map!(
        pair!(
            color_head,
            terminated!(many0!(call!(segment, "[/color]")),
                        tag_no_case!("[/color]"))
        ),
        |((r, g, b), text)| Segment::Decorated {
            style: DecorationStyle::Color(r, g, b),
            text,
        }
    )
);

named!(color_head(&str) -> (u8, u8, u8),
    delimited!(
        tag_no_case!("[color="),
        alt!(rgb_color | css_color),
        char!(']')
    )
);

named!(rgb_color(&str) -> (u8, u8, u8),
    map!(
        preceded!(
            char!('#'),
            verify!(nom::hex_digit1, |d: &str| d.len() == 3 || d.len() == 6)
        ),
        |digits| {
            if digits.len() == 3 {
                let (r, g, b) = (
                    u8::from_str_radix(&digits[0..1], 16).unwrap() << 4,
                    u8::from_str_radix(&digits[1..2], 16).unwrap() << 4,
                    u8::from_str_radix(&digits[2..3], 16).unwrap() << 4
                );
                (r + (r << 4),
                 g + (g << 4),
                 b + (b << 4))
            } else {
                (u8::from_str_radix(&digits[0..2], 16).unwrap(),
                 u8::from_str_radix(&digits[2..4], 16).unwrap(),
                 u8::from_str_radix(&digits[4..6], 16).unwrap())
            }
        }
    )
);

named!(css_color(&str) -> (u8, u8, u8),
    map!(
        verify!(
            map!(nom::alphanumeric1, palette::named::from_str),
            |c: Option<Srgb<u8>>| c.is_some()
        ),
        |c| c.unwrap().into_components()
    )
);
