use std::collections::HashMap;

use crate::{
    cssom::{
        CSSDeclaration, CSSProperty, CSSRule, CSSSelector, CSSSpecifity, CSSValue, SimpleSelector,
        Stylesheet,
    },
    dom::{self, DomNode, ElementData, NodeType, TagType},
};

pub type PropertyMap = HashMap<CSSProperty, CSSValue>;

#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node: Box<DomNode>,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode>,
}

#[derive(Debug, PartialEq)]
pub enum Display {
    Block,
    Inline,
    None,
}

impl StyledNode {
    pub fn get_computed_value(&self, name: &CSSProperty) -> Option<CSSValue> {
        match self.specified_values.get(name) {
            Some(v) => Some((*v).clone()),
            None => Some(name.default_value()),
        }
    }

    pub fn get_computed_display(&self) -> Display {
        if let Some(TagType::Span) = self.node.get_tag_type() {
            return Display::Inline;
        };

        match self.get_computed_value(&CSSProperty::Display) {
            Some(CSSValue::Keyword(value)) if value == "inline" => Display::Inline,
            Some(CSSValue::Keyword(value)) if value == "none" => Display::None,
            value => Display::Block,
        }
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    if selector.tag.iter().any(|name| elem.tag_type != *name) {
        return false;
    }

    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    let elem_classes = elem.classes();
    if selector
        .class
        .iter()
        .any(|class| !elem_classes.contains(class.as_str()))
    {
        return false;
    }

    return true;
}

fn matches(node: &ElementData, selector: &CSSSelector) -> bool {
    match selector {
        CSSSelector::SimpleSelector(selector) => matches_simple_selector(node, &selector),
    }
}

fn matches_rule(node: &ElementData, rule: &CSSRule) -> Option<CSSSpecifity> {
    let mut matched_rules: Vec<CSSSpecifity> = rule
        .selectors
        .iter()
        .filter(|selector| matches(node, selector))
        .map(|selector| selector.specificity())
        .collect();
    matched_rules.sort_by(|a, b| b.cmp(a));
    return matched_rules.iter().next().copied();
}

fn get_specified_values(node: &DomNode, stylesheet: &Stylesheet) -> PropertyMap {
    if let NodeType::Text(_) = &node.get_node_type() {
        return HashMap::new();
    }

    let NodeType::Element(element) = &node.get_node_type() else {
        unreachable!();
    };
    match element.tag_type {
        dom::TagType::Style => HashMap::new(),
        _ => {
            let mut matched_rules: Vec<(CSSSpecifity, &CSSRule)> = stylesheet
                .rules
                .iter()
                .map(|rule| (matches_rule(element, rule), rule))
                .filter_map(|x| match x {
                    (Some(specificity), rule) => Some((specificity, rule)),
                    (None, _) => None,
                })
                .collect();

            matched_rules.sort_by(|a, b| a.0.cmp(&b.0));
            let mut specified_values = HashMap::new();
            let mut specified_is_important: HashMap<CSSProperty, bool> = HashMap::new();
            for (_, rule) in matched_rules {
                for CSSDeclaration {
                    property,
                    value,
                    is_important,
                } in &rule.declarations
                {
                    if specified_is_important.contains_key(property)
                        && !is_important
                        && specified_is_important[property]
                    {
                        continue;
                    }

                    specified_values.insert(*property, value.clone());
                    specified_is_important.insert(*property, *is_important);
                }
            }
            specified_values
        }
    }
}

pub fn generate_styled_node(node: &DomNode, stylesheet: &Stylesheet) -> StyledNode {
    StyledNode {
        specified_values: get_specified_values(&node, &stylesheet),
        children: node
            .get_children()
            .into_iter()
            .map(|child| generate_styled_node(child, &stylesheet))
            .collect(),
        node: Box::new(node.clone()),
    }
}

mod tests {
    use crate::{
        cssom::{CSSProperty, CSSValue},
        parser::{CSSParser, HTMLParser, IParser},
        style::{generate_styled_node, Display},
    };

    #[test]
    fn test_generated_styled_tree() {
        let html = "
            <div class='my-div'>
                Hello world!
            </div>
        ";
        let css = "
            div {
                color: #fff;
            }

            html {
                color: #000;
            }
        ";
        let stylesheet = CSSParser::new(css).parse();
        let dom = HTMLParser::new(html).parse();
        let styled_dom = generate_styled_node(&dom, &stylesheet);
        let Some(CSSValue::Keyword(val)) = styled_dom.specified_values.get(&CSSProperty::Color)
        else {
            panic!("CSS rule was not applied to HTML tag")
        };
        assert_eq!(val, "#000");
        let Some(CSSValue::Keyword(val)) = styled_dom.children[0]
            .specified_values
            .get(&CSSProperty::Color)
        else {
            panic!("CSS rule was not applied to DIV tag")
        };
        assert_eq!(val, "#fff");
    }

    #[test]
    fn test_display_none() {
        let html = "<div style='display: none'>Hidden</div>";
        let css = "div { display: none; }";
        let stylesheet = CSSParser::new(css).parse();
        let dom = HTMLParser::new(html).parse();
        let styled_dom = generate_styled_node(&dom, &stylesheet);

        assert!(matches!(
            styled_dom.children[0].get_computed_display(),
            Display::None
        ));
    }

    #[test]
    fn test_inline_block_display() {
        let html = "<div>Inline text</div>";
        let css = "div { display: inline; }";
        let stylesheet = CSSParser::new(css).parse();
        let dom = HTMLParser::new(html).parse();
        let styled_dom = generate_styled_node(&dom, &stylesheet);

        assert_eq!(
            styled_dom.children[0].get_computed_display(),
            Display::Inline
        );
    }

    #[test]
    fn test_css_specificity_ordering() {
        let html = "<p class='foo'>Text</p>";
        let css = "
            p { color: red; }
            .foo { color: blue; }
        ";
        let stylesheet = CSSParser::new(css).parse();
        let dom = HTMLParser::new(html).parse();
        let styled_dom = generate_styled_node(&dom, &stylesheet);

        let Some(CSSValue::Keyword(val)) =
            styled_dom.children[0].get_computed_value(&CSSProperty::Color)
        else {
            panic!("CSS rule was not applied")
        };
        assert_eq!(val, "blue");
    }

    #[test]
    fn test_style_element_by_id() {
        let html = "<div id='test'>Hello</div>";
        let css = "#test { color: green; }";
        let stylesheet = CSSParser::new(css).parse();
        let dom = HTMLParser::new(html).parse();
        let styled_dom = generate_styled_node(&dom, &stylesheet);

        let Some(CSSValue::Keyword(val)) =
            styled_dom.children[0].get_computed_value(&CSSProperty::Color)
        else {
            panic!("CSS rule was not applied")
        };
        assert_eq!(val, "green");
    }
}
