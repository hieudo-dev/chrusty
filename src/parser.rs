use std::collections::HashMap;
use std::marker::PhantomData;

use crate::cssom::{
    new_css_declaration, new_css_rule, new_css_selector, CSSDeclaration, CSSProperty, CSSRule,
    CSSSelector, CSSValue, ColorData, Stylesheet, Unit,
};
use crate::dom;

pub trait IParser {
    fn next_char(&self) -> char;

    fn next_char_at(&self, offset: usize) -> char;

    fn eof(&self) -> bool;

    fn consume_char(&mut self) -> Result<char, &str>;

    fn starts_with(&self, s: &str) -> bool;

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool;

    fn consume_white_space(&mut self);
}

#[derive(Debug)]
pub struct Parser<'i, I> {
    pos: usize,
    input: &'i str,
    marker: PhantomData<I>,
}

pub struct Css;
pub struct Html;

impl<I> IParser for Parser<'_, I> {
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn next_char_at(&self, offset: usize) -> char {
        self.input[(self.pos + offset)..].chars().next().unwrap()
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> Result<char, &str> {
        if self.eof() {
            return Err("All input characters already consumed");
        }

        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_post, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_post;
        return Ok(cur_char);
    }

    fn starts_with(&self, s: &str) -> bool {
        &self.input[self.pos..] == s
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char().unwrap())
        }
        return result;
    }

    fn consume_white_space(&mut self) {
        self.consume_while(char::is_whitespace);
    }
}

impl<'i> Parser<'i, Css> {
    pub fn new(input: &'i str) -> Self {
        Self {
            pos: 0,
            input,
            marker: PhantomData,
        }
    }

    pub fn parse(&mut self) -> Stylesheet {
        let mut stylesheet = Stylesheet::new(vec![]);
        self.consume_white_space();
        while !self.eof() {
            let rule = self.parse_rule();
            stylesheet.add_rule(rule);
            self.consume_white_space();
        }
        stylesheet
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(|chr| {
            chr != '.' && chr != '#' && chr != '{' && chr != '}' && chr != ':' && chr != ';'
        })
    }

    fn parse_rule(&mut self) -> CSSRule {
        let selector = self.parse_selector();
        assert_eq!(self.consume_char(), Ok('{'));
        let declarations = self.parse_declarations();
        self.consume_white_space();
        assert_eq!(self.consume_char(), Ok('}'));
        return new_css_rule(selector, declarations);
    }

    fn parse_selector(&mut self) -> CSSSelector {
        self.consume_white_space();
        let mut class: Vec<String> = vec![];
        let mut id: Option<String> = None;
        let tag: Option<String> = match self.next_char() {
            '.' | '#' => None,
            _ => Some(self.consume_while(|c| c != '.' && c != '#' && c != '{')),
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    let _ = self.consume_char();
                    id = Some(self.parse_identifier());
                }
                '.' => {
                    let _ = self.consume_char();
                    class.push(self.parse_identifier())
                }
                _ => break,
            }
        }
        return new_css_selector(tag, class, id);
    }

    fn parse_property(&mut self) -> CSSProperty {
        self.consume_white_space();
        let prop_name = self.parse_identifier();
        return match prop_name.as_ref() {
            "background" => CSSProperty::Background,
            "width" => CSSProperty::Width,
            "height" => CSSProperty::Height,
            "color" => CSSProperty::Color,
            x => panic!("Following CSS property is not supported: {}", x),
        };
    }

    fn parse_value(&mut self) -> CSSValue {
        self.consume_white_space();
        return {
            if self.starts_with("rgb(") {
                self.consume_while(|c| c != '(');
                assert_eq!(self.consume_char(), Ok('('));
                let r = self.consume_while(char::is_numeric).parse::<u32>().unwrap();
                assert_eq!(self.consume_char(), Ok(','));
                let g = self.consume_while(char::is_numeric).parse::<u32>().unwrap();
                assert_eq!(self.consume_char(), Ok(','));
                let b = self.consume_while(char::is_numeric).parse::<u32>().unwrap();
                assert_eq!(self.consume_char(), Ok(')'));
                return CSSValue::Color(ColorData::Rgb(r, g, b));
            } else if char::is_numeric(self.next_char()) {
                let value = self
                    .consume_while(|c| c != 'p' && c != '%')
                    .parse::<f32>()
                    .unwrap();
                let unit = {
                    let unit = self.consume_while(|c| c != ';');
                    match unit.as_str() {
                        "%" => Unit::Percent,
                        _ => Unit::Px,
                    }
                };
                return CSSValue::Dimension(value, unit);
            } else {
                let value = self.consume_while(|c| c != ';');
                CSSValue::Keyword(value)
            }
        };
    }

    fn parse_declarations(&mut self) -> Vec<CSSDeclaration> {
        let mut declarations: Vec<CSSDeclaration> = vec![];
        self.consume_white_space();
        while self.next_char() != '}' {
            let property = self.parse_property();
            self.consume_white_space();
            assert_eq!(self.consume_char(), Ok(':'));
            let value = self.parse_value();
            self.consume_white_space();
            let important = self.consume_while(|x| x != ';');
            let is_important = match important.trim() {
                "!important" => true,
                _ => false,
            };
            assert_eq!(self.consume_char(), Ok(';'));
            declarations.push(new_css_declaration(property, value, is_important));
            self.consume_white_space();
        }
        return declarations;
    }
}

impl<'i> Parser<'i, Html> {
    pub fn new(input: &'i str) -> Self {
        Self {
            pos: 0,
            input,
            marker: PhantomData,
        }
    }

    pub fn parse(&mut self) -> dom::Document {
        dom::Document {
            children: self.parse_nodes(),
        }
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
