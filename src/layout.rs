use crate::{
    cssom::{CSSProperty, CSSValue},
    style::{Display, StyledNode},
};

#[derive(Debug, Default)]
struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

#[derive(Default, Debug)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Default)]
struct Dimensions {
    boundingRect: Rect,
    padding: EdgeSizes,
    content: Rect,
    // margin: EdgeSizes,
}

#[derive(Debug)]
enum BoxType<'a> {
    Block(&'a StyledNode<'a>),
    Inline(&'a StyledNode<'a>),
    Anonymous,
}

#[derive(Debug)]
pub struct LayoutBox<'a> {
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}

impl<'a> LayoutBox<'a> {
    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::Inline(_) | BoxType::Anonymous => self,
            BoxType::Block(_) => {
                match &self.children.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::Anonymous,
                        ..
                    }) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::Anonymous)),
                }
                self.children.last_mut().unwrap()
            }
        }
    }

    fn new(box_type: BoxType<'a>) -> LayoutBox<'a> {
        LayoutBox {
            dimensions: Default::default(),
            box_type,
            children: vec![],
        }
    }

    fn get_styled_node(&self) -> &StyledNode {
        match self.box_type {
            BoxType::Block(x) | BoxType::Inline(x) => x,
            _ => panic!("Box type unsupported: {:#?}", self.box_type),
        }
    }

    pub fn layout(&mut self, container: &Dimensions) {
        match self.box_type {
            BoxType::Block(_) => self.layout_block(container),
            BoxType::Inline(_) => todo!(),
            BoxType::Anonymous => todo!(),
        }
    }

    fn layout_block(&mut self, container: &Dimensions) {
        self.layout_block_width(container);

        self.layout_block_position(container);
        self.layout_block_children(container);
        self.layout_block_height(container);
    }

    fn layout_block_width(&mut self, container: &Dimensions) {
        let style = self.get_styled_node();

        let padding = style.get_computed_value(&CSSProperty::Padding);
        // TODO: add support unit types
        let Some(CSSValue::Dimension(paddingValue, _)) = padding else {
            panic!("Padding value unsupported: {}", padding.unwrap());
        };

        let width = style.get_computed_value(&CSSProperty::Width);
        let Some(CSSValue::Dimension(widthValue, _)) = width else {
            panic!("Width value unsupported: {}", width.unwrap());
        };

        self.dimensions.padding.left = paddingValue;
        self.dimensions.padding.right = paddingValue;
        self.dimensions.boundingRect.width = widthValue;
    }

    fn layout_block_position(&mut self, container: &Dimensions) {
        let style = self.get_styled_node();
        let padding = style.get_computed_value(&CSSProperty::Padding);
        let Some(CSSValue::Dimension(paddingValue, _)) = padding else {
            panic!("Padding value unsupported: {}", padding.unwrap());
        };
        self.dimensions.padding.top = paddingValue;
        self.dimensions.padding.bottom = paddingValue;

        self.dimensions.boundingRect.x = container.boundingRect.x + container.padding.left;
        self.dimensions.boundingRect.y =
            container.boundingRect.y + container.padding.top + container.content.height;
    }

    fn layout_block_children(&mut self, container: &Dimensions) {
        for child in &mut self.children {
            child.layout(&self.dimensions);
            self.dimensions.content.height += child.dimensions.boundingRect.height;
        }
    }

    fn layout_block_height(&mut self, container: &Dimensions) {
        self.dimensions.boundingRect.height = self.dimensions.padding.top
            + self.dimensions.content.height
            + self.dimensions.padding.bottom;
    }
}

pub fn build_layout_tree<'a>(styled_node: &'a StyledNode) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(match styled_node.get_computed_display() {
        Display::Block => BoxType::Block(&styled_node),
        Display::Inline => BoxType::Inline(&styled_node),
        Display::None => panic!("Root has diplay none"),
    });

    for child in &styled_node.children {
        match child.get_computed_display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root
                .get_inline_container()
                .children
                .push(build_layout_tree(child)),
            Display::None => {}
        }
    }

    return root;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cssom::{
        new_css_declaration, new_css_rule, new_css_selector, CSSProperty, CSSRule, CSSValue,
        Stylesheet, Unit,
    };
    use crate::dom::{new_element, IDomNode, Node, NodeType, TagType};
    use crate::style::{generate_styled_node, Display, StyledNode};
    use std::collections::HashMap;

    #[test]
    fn create_layout_box() {
        let node = new_element(TagType::Div, HashMap::new(), vec![]);
        let stylesheet = Stylesheet::default();
        let styled_node = generate_styled_node(&node, &stylesheet);
        let layout_box = LayoutBox::new(BoxType::Block(&styled_node));
        assert!(matches!(layout_box.box_type, BoxType::Block(_)));
    }

    #[test]
    fn get_inline_container_creates_anonymous_box() {
        let node = new_element(TagType::Div, HashMap::new(), vec![]);
        let stylesheet = Stylesheet::default();
        let styled_node = generate_styled_node(&node, &stylesheet);
        let mut layout_box = LayoutBox::new(BoxType::Block(&styled_node));
        let inline_container = layout_box.get_inline_container();
        assert!(matches!(inline_container.box_type, BoxType::Anonymous));
    }

    #[test]
    fn layout_block_sets_correct_dimensions() {
        let node = new_element(TagType::Div, HashMap::new(), vec![]);
        let mut stylesheet = Stylesheet::new(vec![]);
        stylesheet.add_rule(new_css_rule(
            vec![new_css_selector(Some(TagType::Div), vec![], None)],
            vec![
                new_css_declaration(
                    CSSProperty::Padding,
                    CSSValue::Dimension(10.0, Unit::Px),
                    false,
                ),
                new_css_declaration(
                    CSSProperty::Width,
                    CSSValue::Dimension(100.0, Unit::Px),
                    false,
                ),
            ],
        ));
        let styled_node = generate_styled_node(&node, &stylesheet);
        let mut layout_box = LayoutBox::new(BoxType::Block(&styled_node));
        let container = Dimensions::default();
        layout_box.layout(&container);
        assert_eq!(layout_box.dimensions.padding.left, 10.0);
        assert_eq!(layout_box.dimensions.padding.right, 10.0);
        assert_eq!(layout_box.dimensions.padding.bottom, 10.0);
        assert_eq!(layout_box.dimensions.padding.top, 10.0);
        assert_eq!(layout_box.dimensions.boundingRect.width, 100.0);
    }

    #[test]
    fn build_layout_tree_constructs_correct_hierarchy() {
        let node = new_element(
            TagType::Div,
            HashMap::new(),
            vec![new_element(TagType::Span, HashMap::new(), vec![])],
        );
        let stylesheet = Stylesheet::default();
        let root_node = generate_styled_node(&node, &stylesheet);
        let layout_tree = build_layout_tree(&root_node);
        assert_eq!(layout_tree.children.len(), 1);
        assert!(matches!(
            layout_tree.children[0].box_type,
            BoxType::Anonymous
        ));
    }
}
