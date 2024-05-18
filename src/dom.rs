use std::{
    collections::{HashMap, HashSet},
    fmt,
};

pub trait IDomNode {
    fn get_children(&self) -> &Vec<Node>;
    fn get_node_type(&self) -> &NodeType;
}

#[derive(Debug)]
pub struct Document {
    pub children: Vec<Node>,
    pub node_type: NodeType,
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in &self.children {
            child.recursive_fmt(f, 0);
        }
        Ok(())
    }
}

impl IDomNode for Document {
    fn get_children(&self) -> &Vec<Node> {
        return &self.children;
    }

    fn get_node_type(&self) -> &NodeType {
        return &self.node_type;
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

impl IDomNode for Node {
    fn get_children(&self) -> &Vec<Node> {
        return &self.children;
    }

    fn get_node_type(&self) -> &NodeType {
        return &self.node_type;
    }
}

#[derive(Debug)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
}

#[derive(Debug)]
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

#[derive(Debug, PartialEq)]
pub enum TagType {
    Html,
    Div,
    P,
    Style,
}

impl std::fmt::Display for TagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagType::Html => write!(f, "html"),
            TagType::Div => write!(f, "div"),
            TagType::P => write!(f, "p"),
            TagType::Style => write!(f, "style"),
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
