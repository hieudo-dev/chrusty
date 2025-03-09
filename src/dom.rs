use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug},
};

#[derive(Debug, Clone)]
pub struct DomNode {
    node_type: NodeType,
    children: Vec<DomNode>,
}

impl fmt::Display for DomNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in &self.children {
            child.recursive_fmt(f, 0);
        }
        Ok(())
    }
}

impl DomNode {
    fn recursive_fmt(&self, f: &mut fmt::Formatter<'_>, depth: usize) {
        let indent_root = "\t".repeat(depth);
        match &self.node_type {
            NodeType::Element(element) => {
                write!(f, "{}<{}", indent_root, element.tag_type);
                for i in &element.attributes {
                    let (key, val) = i;
                    write!(f, " {}='{}'", key, val);
                }
                write!(f, ">\n");
                for child in &self.children {
                    child.recursive_fmt(f, depth + 1);
                }
                write!(f, "{}</{}>\n", indent_root, element.tag_type);
            }
            NodeType::Text(content) => {
                write!(f, "{}{}\n", indent_root, content);
            }
        }
    }

    pub fn new(node_type: NodeType, children: Vec<DomNode>) -> DomNode {
        DomNode {
            node_type,
            children,
        }
    }

    pub fn get_children(&self) -> &Vec<DomNode> {
        return &self.children;
    }

    pub fn get_node_type(&self) -> &NodeType {
        return &self.node_type;
    }

    pub fn get_tag_type(&self) -> Option<TagType> {
        match &self.node_type {
            NodeType::Text(_) => None,
            NodeType::Element(ElementData {
                tag_type,
                attributes,
            }) => Some((*tag_type)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
}

#[derive(Debug, Clone)]
pub struct ElementData {
    pub tag_type: TagType,
    pub attributes: HashMap<String, String>,
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TagType {
    Html,
    Div,
    P,
    Span,
    Style,
}

impl std::fmt::Display for TagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagType::Html => write!(f, "html"),
            TagType::Div => write!(f, "div"),
            TagType::Span => write!(f, "span"),
            TagType::P => write!(f, "p"),
            TagType::Style => write!(f, "style"),
        }
    }
}

type AttrsMap = HashMap<String, String>;

pub fn new_text(content: &str, children: Vec<DomNode>) -> DomNode {
    DomNode {
        children,
        node_type: NodeType::Text(String::from(content.trim())),
    }
}

pub fn new_element(tag_type: TagType, attributes: AttrsMap, children: Vec<DomNode>) -> DomNode {
    DomNode {
        children,
        node_type: NodeType::Element(ElementData {
            tag_type,
            attributes,
        }),
    }
}
