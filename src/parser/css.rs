use crate::{
    cssom::{
        new_css_declaration, new_css_rule, new_css_selector, CSSDeclaration, CSSProperty, CSSRule,
        CSSSelector, CSSValue, ColorData, Stylesheet, Unit,
    },
    parser::{ICharStreamParser, IParser},
};

#[derive(Debug)]
pub struct CSSParser {
    pos: usize,
    input: String,
}
impl_CharStream!(for CSSParser);

impl CSSParser {
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

impl IParser for CSSParser {
    type Output = Stylesheet;

    fn new(input: &str) -> CSSParser {
        CSSParser {
            pos: 0,
            input: String::from(input),
        }
    }
    fn parse(&mut self) -> Self::Output {
        let mut stylesheet = Stylesheet::new(vec![]);
        self.consume_white_space();
        while !self.eof() {
            let rule = self.parse_rule();
            stylesheet.add_rule(rule);
            self.consume_white_space();
        }
        stylesheet
    }
}
