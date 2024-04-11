use std::{collections::HashMap, fmt};

#[derive(Debug)]
pub struct Document {
    pub children: Vec<Node>,
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in &self.children {
            child.recursive_fmt(f, 0);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Node {
    children: Vec<Node>,
    node_type: NodeType,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.recursive_fmt(f, 0);
        Ok(())
    }
}

impl Node {
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
}

#[derive(Debug)]
enum NodeType {
    // TODO: Add support for more Node types
    Text(String),
    Element(ElementData),
}

#[derive(Debug)]
struct ElementData {
    tag_type: TagType,
    attributes: HashMap<String, String>,
}

#[derive(Debug)]
pub enum TagType {
    Div,
    P,
}

impl std::fmt::Display for TagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagType::Div => write!(f, "div"),
            TagType::P => write!(f, "p"),
        }
    }
}

type AttrsMap = HashMap<String, String>;

pub fn new_text(content: &str, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Text(String::from(content.trim())),
    }
}

pub fn new_element(tag_type: TagType, attributes: AttrsMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData {
            tag_type,
            attributes,
        }),
    }
}
