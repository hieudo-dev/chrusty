use std::collections::HashMap;

use crate::{
    cssom::{
        CSSDeclaration, CSSProperty, CSSRule, CSSSelector, CSSSpecifity, CSSValue, SimpleSelector,
        Stylesheet,
    },
    dom::{self, ElementData, IDomNode, NodeType},
};

type PropertyMap<'a> = HashMap<&'a CSSProperty, &'a CSSValue>;

pub struct StyledNode<'a> {
    node: &'a dyn IDomNode,
    specified_values: PropertyMap<'a>,
    children: Vec<StyledNode<'a>>,
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
        .any(|class| !elem_classes.contains(&**class))
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
    matched_rules.sort_by(|a, b| b.cmp(&a));
    matched_rules.iter().next().copied()
}

fn get_specified_values<'a>(node: &dyn IDomNode, stylesheet: &'a Stylesheet) -> PropertyMap<'a> {
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
            let mut specified_values: HashMap<&'a CSSProperty, &'a CSSValue> = HashMap::new();
            let mut specified_is_important: HashMap<&'a CSSProperty, bool> = HashMap::new();
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

                    specified_values.insert(property, value);
                    specified_is_important.insert(property, *is_important);
                }
            }
            specified_values
        }
    }
}

pub fn get_styled_node<'a>(node: &'a dyn IDomNode, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: node,
        specified_values: get_specified_values(node, stylesheet),
        children: node
            .get_children()
            .iter()
            .map(|child| get_styled_node(child, stylesheet))
            .collect(),
    }
}

mod tests {
    use crate::{
        cssom::{CSSProperty, CSSValue},
        parser::{CSSParser, HTMLParser, IParser},
        style::get_styled_node,
    };

    #[test]
    fn generates_styled_tree() {
        let html = "
            <div class=\"my-div\">
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
        let styled_dom = get_styled_node(&dom, &stylesheet);
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
}
