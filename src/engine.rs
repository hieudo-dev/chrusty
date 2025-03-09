use crate::{
    dom::DomNode,
    layout::{generate_layout_tree, LayoutBox},
    parser::{CSSParser, HTMLParser, IParser},
    style::{generate_styled_node, StyledNode},
};

pub fn parse_to_layout<'a>(html: &str, css: &str) -> LayoutBox {
    let stylesheet = CSSParser::new(css).parse();
    let dom = HTMLParser::new(html).parse();
    let styled_dom = generate_styled_node(&dom, &stylesheet);
    let layout_tree = generate_layout_tree(&styled_dom);
    return layout_tree;
}
