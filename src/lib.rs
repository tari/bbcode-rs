//! A parsing library for [bbcode](https://en.wikipedia.org/wiki/BBCode)
//!
//! Unlike most bbcode "parser"s, this constructs a syntax tree that is
//! easy to transform to a custom representation or analyze for other
//! purposes. Its goal is primarily customizability of the output.
//!
//! Getting the syntax tree for textual bbcode is as easy as calling
//! `parse`:
//!
//! ```
//! use bbcode::{parse, DecorationStyle, Segment};
//!
//! static MESSAGE: &'static str =
//! r#"[quote="Batman"]I'm batman[/quote]
//! Isn't he [i]dreamy[/i]?"#;
//!
//! let ast = parse(MESSAGE);
//! assert_eq!(ast, vec![
//!     Segment::Quote {
//!         attribution: Some("Batman"),
//!         body: vec![Segment::Text("I'm batman")],
//!     },
//!     Segment::Text("\nIsn't he "),
//!     Segment::Decorated {
//!         style: DecorationStyle::Italic,
//!         text: vec![Segment::Text("dreamy")],
//!     },
//!     Segment::Text("?"),
//! ]);
//! ```
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate palette;

use nom::{types::CompleteStr, AtEof};
use std::fmt::Debug;
use std::os::raw::c_char;

#[macro_use]
mod macros;
mod code;
mod decoration;
mod list;
mod quote;
pub mod render;
mod url;

pub use decoration::DecorationStyle;
pub use list::ListStyle;

/// FFI entry point; converts a UTF-8 string of bbcode to rendered code.
///
/// If the input string contains any invalid UTF-8 bytes it will be
/// recovered by insertion of U+FFFD REPLACEMENT CHARACTER.
///
/// The returned pointer must be freed by calling `bbcode_dispose`.
#[no_mangle]
pub extern "C" fn bbcode_translate(s: *const c_char) -> *mut c_char {
    use std::ffi::{CStr, CString};
    use render::Renderer;

    let utf8 = unsafe {
        CStr::from_ptr(s).to_string_lossy()
    };
    let segments = parse(&utf8);

    // Render into a memory buffer; we're likely to emit about as many bytes
    // as there are in the input, maybe more.
    let mut buf = Vec::<u8>::with_capacity(utf8.len());
    {
        let mut renderer = render::SimpleHtml::new(&mut buf);
        renderer.render(&segments)
            .expect("Rendering to a memory buffer should never fail");
    }
    return CString::new(buf)
        .expect("Rendering should not generate null bytes")
        .into_raw();
}

/// Free a string returned from `bbcode_translate`.
#[no_mangle]
pub extern "C" fn bbcode_dispose(s: *mut c_char) {
    use std::ffi::CString;

    let _ = unsafe {
        CString::from_raw(s)
    };
}

/// Any logical segment of data- a tag or plain text.
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Segment<'a> {
    /// Unadorned text.
    Text(&'a str),
    /// A span of text with simple decoration.
    Decorated {
        style: DecorationStyle,
        text: Vec<Segment<'a>>,
    },
    /// A blockquote with a body and optional attribution.
    Quote {
        attribution: Option<&'a str>,
        body: Vec<Segment<'a>>,
    },
    /// A block of code, displayed verbatim.
    Code(&'a str),
    /// A list of items with a specified style.
    List {
        style: ListStyle,
        items: Vec<Vec<Segment<'a>>>,
    },
    /// A hyperlink.
    Link {
        // The target of the hyperlink (`href` attribute for HTML `a`).
        target: &'a str,
        text: Vec<Segment<'a>>,
    },
    /// A picture, displayed inline.
    Image { src: &'a str }, // TODO extra items
                            // [youtube]
                            // [hr]
                            // [h1] - [h6]
                            // [sub]
                            // [sup]
                            // [strike]
                            // [mono]
}

/// Parse a string into a sequence of `Segment`s.
pub fn parse(s: &str) -> Vec<Segment> {
    _parse(CompleteStr(s))
}

fn _parse(mut input: CompleteStr) -> Vec<Segment> {
    let mut out = Vec::new();
    loop {
        if input.at_eof() && input.is_empty() {
            break;
        }

        match segment(&input, ()) {
            Ok((tail, s)) => {
                out.push(s);
                input = CompleteStr(tail);
            }
            e => panic!("segment() should not fail but did: {:?}", e),
        }
    }
    out
}

/// Strings that can mark the end of a segment.
///
/// This trait is used by `text_segment` (and thus also `segment`) to determine
/// where a given freeform (coded or not) segment ends when inside another coded
/// segment.
///
/// When not required, pass `()`; the impl for unit never matches.
trait Terminal {
    /// Return true if `s` begins with a substring that matches `self`.
    fn leads(&self, s: &str) -> bool;
}

/// Never matches the input; appropriate for unbounded segments.
impl Terminal for () {
    fn leads(&self, _s: &str) -> bool {
        false
    }
}

/// Simple leading substring match, case-insensitive ASCII-only.
impl Terminal for &str {
    fn leads(&self, s: &str) -> bool {
        if s.len() < self.len() {
            // Slice on RangeTo fails if end index exceeds len,
            // so short-circuit this case.
            return false;
        }
        self.eq_ignore_ascii_case(&s[..self.len()])
    }
}

/// Leading substring match for any item.
impl Terminal for &[&str; 2] {
    fn leads(&self, s: &str) -> bool {
        self.iter().any(|p| p.leads(s))
    }
}

fn segment<'a, T: Terminal + Debug>(
    input: &'a str,
    terminal: T,
) -> nom::IResult<&'a str, Segment<'a>, u32> {
    // Try to find a valid coded segment, otherwise take plain text.
    //
    // TODO this could be converted to support streaming; it returns either
    // a coded segment, eof or incomplete- a leading coded segment can be
    // returned, and a text segment can be returned if the tail's length is
    // nonzero or the input is at eof. This means in the pure-text case we
    // still need to buffer the entire input, but can opportunistically stream
    // segments as they appear.
    if let Ok((tail, segment)) = coded_segment(input) {
        return Ok((tail, segment));
    }
    let (tail, text) = text_segment(input, terminal)?;
    return Ok((tail, Segment::Text(text)));
}

named!(coded_segment(&str) -> Segment,
    alt_complete!(
        decoration::decorated
        | code::code
        | image
        | list::list
        | quote::quote
        | url::url
    )
);

/// Matches any nonzero amount of input that is not a coded_segment.
///
/// Terminates at end of input or if the provided terminal is found, but does
/// not consume the terminal.
fn text_segment<'a, T: Terminal + Debug>(
    input: &'a str,
    terminal: T,
) -> nom::IResult<&'a str, &'a str, u32> {
    debug!("text_segment until {:?} in {}", terminal, input);

    if terminal.leads(input) {
        return Err(nom::Err::Error(nom::Context::Code(
            input,
            nom::ErrorKind::Custom(0),
        )));
    }

    // Search for either a coded segment or the terminal string, either of which
    // ends the plain text.
    //
    // Text segment applies only if no coded segment was found, so we do not
    // consider the first character of input for coded segments- asking for text
    // implies the caller doesn't expect a coded segment right away.
    //
    // This is required to be expensive- malformed markup should become plain text.
    // The reference implementation does substantially this but the costs are
    // hidden inside regexes.
    //
    // TODO a validation mode that errors out on malformed markup could be useful.
    for (idx, _) in input.char_indices().skip(1) {
        let search_point = &input[idx..];
        if terminal.leads(search_point) || coded_segment(search_point).is_ok() {
            let (consumed, rest) = input.split_at(idx);
            return Ok((rest, consumed));
        }
    }

    // Fell off the end of input without finding a coded segment,
    // so return all of it.
    Ok((&input[input.len()..], input))
}

#[test]
fn text_segment_no_terminal() {
    assert_eq!(text_segment("Hello, world!", ()), Ok(("", "Hello, world!")));
}

#[test]
fn text_segment_with_terminal() {
    assert_eq!(text_segment("Foo\r\nBar", "\r\n"), Ok(("\r\nBar", "Foo")));
}

#[test]
fn nested_segments() {
    let segments = parse("[b]Foo[i]bar[/i][/b]");
    assert_eq!(
        segments,
        vec![Segment::Decorated {
            style: DecorationStyle::Bold,
            text: vec![
                Segment::Text("Foo"),
                Segment::Decorated {
                    style: DecorationStyle::Italic,
                    text: vec![Segment::Text("bar")],
                }
            ],
        }]
    );
}

named!(image(&str) -> Segment,
    map!(
        delimited!(
            tag_no_case!("[img]"),
            take_until_no_case!("[/img]"),
            tag_no_case!("[/img]")
        ),
        |src| Segment::Image { src }
    )
);

#[test]
fn parse_image() {
    assert_eq!(
        parse("[img]http://example.com/foo.webp[/img]"),
        vec![Segment::Image {
            src: "http://example.com/foo.webp"
        }]
    )
}
