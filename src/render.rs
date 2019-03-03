use super::{DecorationStyle, ListStyle, Segment};

pub type Result<E> = std::result::Result<(), E>;

pub trait Renderer {
    type Err;

    fn render(&mut self, segments: &Vec<super::Segment>) -> Result<Self::Err> {
        for segment in segments {
            match segment {
                Segment::Text(s) => self.text(s)?,
                Segment::Decorated {
                    style,
                    text: segments,
                } => {
                    self.decoration_begin(*style)?;
                    self.render(&segments)?;
                    self.decoration_end(*style)?
                }
                Segment::Quote {
                    attribution,
                    body: segments,
                } => {
                    self.quote_begin(attribution)?;
                    self.render(&segments)?;
                    self.quote_end(attribution)?
                }
                Segment::Code(s) => self.code(s)?,
                Segment::List { style, items } => {
                    self.list_begin(*style)?;
                    for item in items {
                        self.list_item_begin(*style)?;
                        self.render(item)?;
                        self.list_item_end(*style)?;
                    }
                    self.list_end(*style)?
                }
                Segment::Link {
                    target,
                    text: segments,
                } => {
                    self.link_begin(target)?;
                    self.render(segments)?;
                    self.link_end(target)?
                }
                Segment::Image { src } => self.image(src)?,
            }
        }

        Ok(())
    }

    /// Output some plain text.
    fn text(&mut self, s: &str) -> Result<Self::Err>;
    /// Output the beginning of a decorated text block.
    fn decoration_begin(&mut self, style: DecorationStyle) -> Result<Self::Err>;
    /// Output the end of a decorated text block.
    fn decoration_end(&mut self, style: DecorationStyle) -> Result<Self::Err>;
    /// Output the beginning of a block quote.
    fn quote_begin(&mut self, attribution: &Option<&str>) -> Result<Self::Err>;
    /// Output the end of a block quote.
    fn quote_end(&mut self, attribution: &Option<&str>) -> Result<Self::Err>;
    /// Output a block of code with contents `s`.
    fn code(&mut self, s: &str) -> Result<Self::Err>;
    /// Output the beginning of a list.
    fn list_begin(&mut self, style: ListStyle) -> Result<Self::Err>;
    /// Output the beginning of a list item.
    fn list_item_begin(&mut self, style: ListStyle) -> Result<Self::Err>;
    /// Output the end of a list item.
    fn list_item_end(&mut self, style: ListStyle) -> Result<Self::Err>;
    /// Output the end of a list.
    fn list_end(&mut self, style: ListStyle) -> Result<Self::Err>;
    /// Output the beginning of a link.
    fn link_begin(&mut self, target: &str) -> Result<Self::Err>;
    /// Output the end of a link.
    fn link_end(&mut self, target: &str) -> Result<Self::Err>;
    fn image(&mut self, src: &str) -> Result<Self::Err>;
}

pub struct SimpleHtml<O>
where
    O: std::io::Write,
{
    out: O,
}

impl<O: std::io::Write> SimpleHtml<O> {
    pub fn new(out: O) -> Self {
        Self { out }
    }

    /// Write s to output, replacing each character in escapes with the corresponding
    /// index of replacements.
    ///
    /// Each escaped character must be one UTF-8 byte (for simplicity) and the
    /// two slices must be the same length.
    fn write_escaped(
        &mut self,
        mut s: &str,
        escapes: &[char],
        replacements: &[&'static str],
    ) -> IoResult<()> {
        debug_assert_eq!(escapes.len(), replacements.len());
        debug_assert!(escapes.iter().all(|c| c.len_utf8() == 1));

        loop {
            let split = match s.find(escapes) {
                Some(i) => i,
                None => break,
            };

            let (head, tail) = s.split_at(split);
            // tail is inclusive of the split point and all of the matched
            // chars are one byte in UTF-8, so taking the first byte here
            // is safe (and easier than pulling out the first char).
            let victim = tail.as_bytes()[0] as char;
            let repl = escapes
                .iter()
                .enumerate()
                .find(|(_, &c)| c == victim)
                .unwrap()
                .0;

            write!(self.out, "{}{}", head, replacements[repl])?;
            s = &tail[1..];
        }

        // Write remaining data past all replaced entities
        write!(self.out, "{}", s)
    }
}

use std::io::Result as IoResult;

impl<O: std::io::Write> Renderer for SimpleHtml<O> {
    type Err = std::io::Error;

    fn text(&mut self, mut s: &str) -> IoResult<()> {
        // Escape tags and entities, also replace newlines with explicit
        // line breaks.
        self.write_escaped(
            s,
            &['&', '<', '>', '\n'],
            &["&amp;", "&lt;", "&gt;", "<br>"],
        )
    }

    fn decoration_begin(&mut self, style: DecorationStyle) -> IoResult<()> {
        use DecorationStyle::*;

        let tag = match style {
            Bold => "b",
            Italic => "i",
            Underline => "u",
            Center => r#"div style="text-align:center""#,
            Color(r, g, b) => {
                return write!(
                    self.out,
                    r#"<span style="color: #{:02x}{:02x}{:02x}">"#,
                    r, g, b
                )
            }
            Size(s) => {
                return write!(self.out, r#"<span style="font-size: {}>"#, s);
            }
        };
        write!(self.out, "<{}>", tag)
    }

    fn decoration_end(&mut self, style: DecorationStyle) -> IoResult<()> {
        use DecorationStyle::*;

        let tag = match style {
            Bold => "b",
            Italic => "i",
            Underline => "u",
            Center => "div",
            Color(..) | Size(..) => "span",
        };
        write!(self.out, "<{}>", tag)
    }

    fn quote_begin(&mut self, attribution: &Option<&str>) -> IoResult<()> {
        if let Some(orig) = attribution {
            write!(self.out, "<div>{} wrote:</div><div>", orig)
        } else {
            write!(self.out, "<div>Quote:</div><div>")
        }
    }

    fn quote_end(&mut self, attribution: &Option<&str>) -> IoResult<()> {
        write!(self.out, "</div>")
    }

    fn code(&mut self, s: &str) -> IoResult<()> {
        write!(self.out, "<pre>")?;
        self.text(s)?;
        write!(self.out, "</pre>")
    }

    fn list_begin(&mut self, style: ListStyle) -> IoResult<()> {
        unimplemented!();
    }

    fn list_item_begin(&mut self, style: ListStyle) -> IoResult<()> {
        unimplemented!();
    }

    fn list_item_end(&mut self, style: ListStyle) -> IoResult<()> {
        unimplemented!();
    }

    fn list_end(&mut self, style: ListStyle) -> IoResult<()> {
        unimplemented!();
    }

    fn link_begin(&mut self, target: &str) -> IoResult<()> {
        unimplemented!();
    }

    fn link_end(&mut self, target: &str) -> IoResult<()> {
        unimplemented!();
    }

    fn image(&mut self, src: &str) -> IoResult<()> {
        write!(self.out, "<img src=\"")?;
        self.write_escaped(src, &['<', '>', '"'], &["&lt;", "&gt;", "&quot;"])?;
        write!(self.out, ">")
    }
}
