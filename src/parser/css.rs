use crate::parser::{ICharStreamParser, IParser};

#[derive(Debug)]
pub struct CSSParser {
    pos: usize,
    input: String,
}
impl_CharStream!(for CSSParser);

impl IParser for CSSParser {
    type Output = String;

    fn new(input: &str) -> CSSParser {
        CSSParser {
            pos: 0,
            input: String::from(input),
        }
    }
    fn parse(&mut self) -> Self::Output {
        panic!("Unimplemented!")
    }
}

impl CSSParser {}
