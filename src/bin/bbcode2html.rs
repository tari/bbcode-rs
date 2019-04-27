extern crate bbcode;

use bbcode::render::{Renderer, SimpleHtml};
use std::io::Read;

fn main() -> Result<(), Box<std::error::Error>> {
    let text = {
        let stdin = std::io::stdin();
        let mut l = stdin.lock();
        let mut s = String::new();
        l.read_to_string(&mut s)?;
        s
    };

    SimpleHtml::new(std::io::stdout()).render(&bbcode::parse(&text))?;

    Ok(())
}
