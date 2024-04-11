use std::collections::HashMap;

use crate::dom::{self, TagType};

#[derive(Debug)]
pub struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    pub fn new(input: &str) -> Parser {
        Parser {
            pos: 0,
            input: String::from(input),
        }
    }

    pub fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    pub fn next_char_at(&self, offset: usize) -> char {
        self.input[(self.pos + offset)..].chars().next().unwrap()
    }

    pub fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    pub fn consume_char(&mut self) -> Result<char, &str> {
        if self.eof() {
            return Err("All input characters already consumed");
        }

        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_post, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_post;
        return Ok(cur_char);
    }

    pub fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char().unwrap())
        }
        return result;
    }

    pub fn consume_white_space(&mut self) {
        self.consume_while(char::is_whitespace);
    }

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

    fn parse_tag(&mut self) -> (TagType, HashMap<String, String>) {
        let _ = self.consume_char();
        let tag = self.consume_while(|c| c != ' ' && c != '>');
        let attributes = self.parse_attributes();
        let _ = self.consume_char();
        let tag_type = match tag.to_lowercase().as_str() {
            "div" => TagType::Div,
            "p" => TagType::P,
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

    pub fn parse(&mut self) -> dom::Document {
        dom::Document {
            children: self.parse_nodes(),
        }
    }
}
