//! Parser utility macros.

/// Like [nom::take_until] but ignores ASCII case.
macro_rules! take_until_no_case (
    ($i:expr, $substr:expr) => (
        {
            use nom::{need_more_err, ErrorKind, InputTake, IResult, Needed};

            let input = $i;
            let target = $substr;
            let mut res: IResult<&str, &str> = need_more_err($i, Needed::Size(target.len()), ErrorKind::TakeUntil);

            for (idx, _) in input.char_indices() {
                println!("Search idx {}: {:?}", idx, &input[idx..]);
                if target.len() > input[idx..].len() {
                    println!("End: target {} shorter than {}", target.len(), input[idx..].len());
                    break;
                }

                let found = target.chars().zip(input[idx..].chars())
                    .all(|(x, y)| x.eq_ignore_ascii_case(&y));
                if found {
                    res = Ok($i.take_split(idx));
                    break;
                }
            }
            res
        }
    );
);

#[test]
fn take_until_no_case() {
    use nom::Err::Incomplete;
    use nom::Needed::Size;

    assert_eq!(
        take_until_no_case!("FooBarBAZbaz", "baz"),
        Ok(("BAZbaz", "FooBar"))
    );
    assert_eq!(take_until_no_case!("BuZz", "BUZZ"), Ok(("BuZz", "")));
    assert_eq!(
        take_until_no_case!("BooFarQuux", "baz"),
        Err(Incomplete(Size(3)))
    );
}

macro_rules! simple_tag (
    ($i:expr, $tag:expr) => (
        delimited!(
            tag_no_case!(concat!("[", $tag, "]")),
            many0!(call!(segment, concat!("[/", $tag, "]"))),
            tag_no_case!(concat!("[/", $tag, "]"))
        )
    );
);
