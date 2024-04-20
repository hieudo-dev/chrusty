use std::fmt::{Display, Formatter, Result};

pub struct Stylesheet {
    rules: Vec<CSSRule>,
}

impl Display for Stylesheet {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for rule in self.rules.iter() {
            write!(f, "{}", rule);
        }
        Ok(())
    }
}

impl Stylesheet {
    pub fn new(rules: Vec<CSSRule>) -> Stylesheet {
        Stylesheet { rules }
    }

    pub fn add_rule(&mut self, rule: CSSRule) {
        self.rules.push(rule)
    }
}

pub struct CSSRule {
    selector: CSSSelector, // TODO: Can upgrade to Vec<CSSSelector> by handling combinators
    declarations: Vec<CSSDeclaration>,
}

impl Display for CSSRule {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} {{\n", self.selector);
        for declaration in self.declarations.iter() {
            write!(f, "\t{}\n", declaration);
        }
        write!(f, "}}\n");
        Ok(())
    }
}

pub enum CSSSelector {
    Simple(SimpleSelector),
}

impl Display for CSSSelector {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            CSSSelector::Simple(SimpleSelector { tag, id, class }) => {
                let tag = match tag {
                    Some(tag) => tag,
                    None => "",
                };
                let id = match id {
                    Some(id) => "#".to_string() + id,
                    None => "".to_string(),
                };
                let class = match class.len() {
                    0 => "".to_string(),
                    _ => ".".to_string() + &class.join("."),
                };
                write!(
                    f,
                    "{}",
                    [tag, id.as_str(), class.as_str()]
                        .into_iter()
                        .filter(|&x| x.len() > 0)
                        .collect::<Vec<&str>>()
                        .join("")
                )
            }
        }
    }
}

struct SimpleSelector {
    tag: Option<String>,
    id: Option<String>,
    class: Vec<String>,
}

#[derive(Debug)]
pub struct CSSDeclaration {
    property: CSSProperty,
    value: CSSValue,
    is_important: bool,
}

impl Display for CSSDeclaration {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let important = match self.is_important {
            true => " !important",
            false => "",
        };
        write!(f, "{}: {}{};", self.property, self.value, important)
    }
}

#[derive(Debug)]
pub enum CSSProperty {
    Background,
    Color,
    Width,
    Height,
}

impl Display for CSSProperty {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let output = match self {
            Self::Background => "background",
            Self::Color => "color",
            Self::Height => "height",
            Self::Width => "width",
        };
        write!(f, "{}", output);
        Ok(())
    }
}

#[derive(Debug)]
pub enum CSSValue {
    Dimension(f32, Unit),
    Keyword(String),
    Color(ColorData),
}

impl Display for CSSValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Dimension(value, unit) => write!(f, "{}{}", value, unit),
            Self::Keyword(kw) => write!(f, "{}", kw),
            Self::Color(data) => match data {
                ColorData::Hex(value) => write!(f, "{}", value),
                ColorData::Rgb(r, g, b) => write!(f, "rgb({}, {}, {})", r, g, b),
            },
        }
    }
}

#[derive(Debug)]
pub enum Unit {
    Px,
    Percent,
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let output = match self {
            Self::Px => "px",
            Self::Percent => "%",
        };
        write!(f, "{}", output);
        Ok(())
    }
}

#[derive(Debug)]
pub enum ColorData {
    Rgb(u32, u32, u32),
    Hex(String),
}

pub fn new_css_rule(selector: CSSSelector, declarations: Vec<CSSDeclaration>) -> CSSRule {
    CSSRule {
        selector,
        declarations,
    }
}

pub fn new_css_declaration(
    property: CSSProperty,
    value: CSSValue,
    is_important: bool,
) -> CSSDeclaration {
    CSSDeclaration {
        property,
        value,
        is_important,
    }
}

pub fn new_css_selector(
    tag: Option<String>,
    class: Vec<String>,
    id: Option<String>,
) -> CSSSelector {
    CSSSelector::Simple(SimpleSelector { tag, id, class })
}
