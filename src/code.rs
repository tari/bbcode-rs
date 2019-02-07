//! Inline code blocks.

use super::Segment;
use super::Segment::Code;

named!(pub code(&str) -> Segment,
    map!(
        delimited!(
            tag_no_case!("[code]"),
            take_until_no_case!("[/code]"),
            tag_no_case!("[/code]")
        ),
        |text| Code(text)
    )
);

#[test]
fn empty_block_ok() {
    assert_eq!(code("[code][/code]"), Ok(("", Code(""))));
}

#[test]
fn takes_text_to_first_close() {
    assert_eq!(
        code("[code]10 PRINT HELLO WORLD\n20 GOTO 10[/code]sup[/code]"),
        Ok(("sup[/code]", Code("10 PRINT HELLO WORLD\n20 GOTO 10")))
    );
}
