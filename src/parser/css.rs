use crate::{
    cssom::{
        new_css_declaration, new_css_rule, new_css_selector, CSSDeclaration, CSSProperty, CSSRule,
        CSSSelector, CSSValue, ColorData, Stylesheet, Unit,
    },
    dom::TagType,
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
            chr != '.'
                && chr != '#'
                && chr != '{'
                && chr != '}'
                && chr != ':'
                && chr != ';'
                && chr != ','
                && !char::is_whitespace(chr)
        })
    }

    fn parse_rule(&mut self) -> CSSRule {
        let selectors = self.parse_selectors();
        assert_eq!(self.consume_char(), Ok('{'));
        let declarations = self.parse_declarations();
        self.consume_white_space();
        assert_eq!(self.consume_char(), Ok('}'));
        return new_css_rule(selectors, declarations);
    }

    fn parse_tag(&mut self) -> Option<TagType> {
        if self.next_char() == '.' || self.next_char() == '#' {
            return None;
        }

        let tag_name =
            self.consume_while(|c| c != '.' && c != '#' && c != '{' && !char::is_whitespace(c));
        return Some(match tag_name.as_ref() {
            "div" => TagType::Div,
            "p" => TagType::P,
            "html" => TagType::Html,
            "style" => TagType::Style,
            tag => panic!("The following tag type is not supported: '{}'", tag),
        });
    }

    fn parse_selectors(&mut self) -> Vec<CSSSelector> {
        let mut selectors: Vec<CSSSelector> = vec![];
        self.consume_white_space();
        while !self.eof() && self.next_char() != '{' {
            let mut class: Vec<String> = vec![];
            let mut id: Option<String> = None;
            let tag: Option<TagType> = self.parse_tag();
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
                    ',' => {
                        let _ = self.consume_char();
                        break;
                    }
                    _ => break,
                }
            }
            selectors.push(new_css_selector(tag, class, id));
            self.consume_white_space();
        }

        return selectors;
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

#[cfg(test)]
mod tests {
    use crate::{
        parser::{CSSParser, IParser},
        utils::minify,
    };

    #[test]
    fn parse() {
        let input = "
            div#id.hello {
                height: 100%;
                background: purple;
                color: #ffffff !important;
            }

            div.my-div,
            div.my-div-2 {
                width: 100px;
                height: 100%;
                background: blue;
                color: #ffffff;
            }

            html {
                background: green;
            }
        ";
        let parsed = CSSParser::new(input).parse();
        assert_eq!(minify(&parsed.to_string()), minify(input))
    }
}
