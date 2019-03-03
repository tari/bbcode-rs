//! Plain text spans with additional decoration.

use super::{segment, Segment};
use palette::Srgb;
use std::num::NonZeroU8;

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
    /// Font size at some arbitrary scale.
    Size(NonZeroU8),
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
        | size
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
                    u8::from_str_radix(&digits[0..1], 16).unwrap(),
                    u8::from_str_radix(&digits[1..2], 16).unwrap(),
                    u8::from_str_radix(&digits[2..3], 16).unwrap()
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

#[test]
fn accepts_colors() {
    assert_eq!(
        color("[color=red]asdf[/color]").unwrap().1,
        Segment::Decorated {
            style: DecorationStyle::Color(255, 0, 0),
            text: vec![Segment::Text("asdf")],
        }
    );

    assert_eq!(
        color("[color=#81f][/color]").unwrap().1,
        Segment::Decorated {
            style: DecorationStyle::Color(0x88, 0x11, 0xFF),
            text: vec![],
        }
    );

    assert_eq!(
        color("[color=#01FE9A]and[/color]").unwrap().1,
        Segment::Decorated {
            style: DecorationStyle::Color(1, 0xFE, 0x9A),
            text: vec![Segment::Text("and")],
        }
    );
}

#[test]
fn rejects_invalid_css_colors() {
    assert!(css_color("beyblade").is_err());
}

named!(pub size(&str) -> Segment,
    map!(
        pair!(
            size_head,
            terminated!(many0!(call!(segment, "[/size]")),
                        tag_no_case!("[/size]"))
        ),
        |(size, text)| Segment::Decorated {
            style: DecorationStyle::Size(size),
            text,
        }
    )
);

named!(size_head(&str) -> NonZeroU8,
    map_opt!(
        verify!(
            map_res!(
                delimited!(
                    tag_no_case!("[size="),
                    nom::digit1,
                    char!(']')
                ),
                str::parse::<u8>
            ),
            |x| x >= 2 && x < 30
        ),
        NonZeroU8::new
    )
);

#[test]
fn enforces_size_limits() {
    assert_eq!(
        size("[size=10]midsize[/size]").unwrap().1,
        Segment::Decorated {
            style: DecorationStyle::Size(NonZeroU8::new(10).unwrap()),
            text: vec![Segment::Text("midsize")],
        }
    );
    assert!(size_head("[size=0]").is_err());
    assert!(size_head("[size=50]").is_err());
}
