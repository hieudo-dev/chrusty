use crate::{
    dom,
    parser::{ICharStreamParser, IParser},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HTMLParser {
    pos: usize,
    input: String,
}
impl_CharStream!(for HTMLParser);

impl IParser for HTMLParser {
    type Output = dom::Document;

    fn new(input: &str) -> HTMLParser {
        HTMLParser {
            pos: 0,
            input: String::from(input),
        }
    }
    fn parse(&mut self) -> dom::Document {
        dom::Document {
            children: self.parse_nodes(),
        }
    }
}

impl HTMLParser {
    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    fn parse_text(&mut self) -> dom::Node {
        dom::new_text(&self.consume_while(|c| c != '<'), vec![])
    }

    fn parse_attributes(&mut self) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        while !self.eof() && self.next_char() != '>' {
            self.consume_white_space();
            let atr_name = self.consume_while(|c| char::is_alphabetic(c) || c == '-');
            assert_eq!(self.consume_char(), Ok('='));
            assert_eq!(self.consume_char(), Ok('"'));
            let atr_value = self.consume_while(|c| c != '"');
            assert_eq!(self.consume_char(), Ok('"'));
            attributes.insert(atr_name, atr_value);
        }
        return attributes;
    }

    fn parse_tag(&mut self) -> (dom::TagType, HashMap<String, String>) {
        let _ = self.consume_char();
        let tag = self.consume_while(|c| c != ' ' && c != '>');
        let attributes = self.parse_attributes();
        let _ = self.consume_char();
        let tag_type = match tag.to_lowercase().as_str() {
            "div" => dom::TagType::Div,
            "p" => dom::TagType::P,
            _ => panic!("The following tag type is not supported: {}", tag),
        };
        return (tag_type, attributes);
    }

    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = vec![];
        loop {
            self.consume_white_space();
            if self.eof() || (self.next_char() == '<' && self.next_char_at(1) == '/') {
                break;
            }

            nodes.push(self.parse_node());
        }
        return nodes;
    }

    fn parse_element(&mut self) -> dom::Node {
        let (tag_type, attributes) = self.parse_tag();
        let children = self.parse_nodes();
        assert_eq!(self.consume_char().unwrap(), '<');
        assert_eq!(self.consume_char().unwrap(), '/');
        self.consume_while(|c| c != '>');
        assert_eq!(self.consume_char().unwrap(), '>');
        dom::new_element(tag_type, attributes, children)
    }
}
