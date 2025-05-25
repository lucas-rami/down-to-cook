use std::str::FromStr;

use super::{
    md_parser::MDError,
    unit::{QuantityOf, Time},
};
use markdown::mdast::Node;

#[derive(Clone, PartialEq)]
pub struct Instructions {
    steps: Vec<Step>,
}

impl Instructions {
    pub fn from_mdast(nodes: &[Node]) -> Result<Self, MDError> {
        match nodes.len() {
            0 => Ok(Self { steps: vec![] }),
            1 => Ok(Self {
                steps: Step::parse_step_list(&nodes[0])?,
            }),
            _ => Err(MDError::new("expected single list node for steps", None)),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Step {
    description: Vec<TextElem>,
    substeps: Vec<Step>,
}

impl Step {
    fn parse_step(node: &Node) -> Result<Self, MDError> {
        match node {
            Node::ListItem(item) => match item.children.len() {
                0 => Ok(Self {
                    description: vec![],
                    substeps: vec![],
                }),
                1 => Ok(Self {
                    description: Self::parse_description(&item.children[0])?,
                    substeps: vec![],
                }),

                2 => Ok(Self {
                    description: Self::parse_description(&item.children[0])?,
                    substeps: Self::parse_step_list(&item.children[1])?,
                }),
                _ => Err(MDError::new(
                    "too many children to list item, expected at most 2",
                    None,
                )),
            },
            _ => Err(MDError::new("expected list item", Some(node))),
        }
    }

    fn parse_description(node: &Node) -> Result<Vec<TextElem>, MDError> {
        match node {
            Node::Paragraph(para) => Ok(para
                .children
                .iter()
                .map(|n| TextElem::from_node(n))
                .collect::<Result<Vec<TextElem>, _>>()?),
            _ => Err(MDError::new("expected paragraph", Some(node))),
        }
    }

    fn parse_step_list(node: &Node) -> Result<Vec<Step>, MDError> {
        match node {
            Node::List(list) => Ok(list
                .children
                .iter()
                .map(|n| Step::parse_step(n))
                .collect::<Result<Vec<Step>, _>>()?),
            _ => Err(MDError::new("expected list", Some(node))),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum TextElem {
    Text(String),
    IngredientRef(String),
    Timer(QuantityOf<Time>),
}

impl TextElem {
    fn from_node(node: &Node) -> Result<Self, MDError> {
        match node {
            Node::Text(text) => Ok(Self::Text(text.value.clone())),
            Node::Emphasis(emphasis) => match emphasis.children.len() {
                0 => Ok(Self::IngredientRef(String::new())),
                1 => match &emphasis.children[0] {
                    Node::Text(text) => Ok(Self::IngredientRef(text.value.clone())),
                    _ => Err(MDError::new(
                        "expected ingrdient ref to be text",
                        Some(&emphasis.children[0]),
                    )),
                },
                _ => Err(MDError::new("expected single children", Some(node))),
            },
            Node::Strong(strong) => match strong.children.len() {
                0 => Ok(Self::IngredientRef(String::new())),
                1 => match &strong.children[0] {
                    Node::Text(text) => match QuantityOf::<Time>::from_str(&text.value[..]) {
                        Ok(quantity) => Ok(Self::Timer(quantity)),
                        Err(_) => Err(MDError::new(
                            &format!("expected time information but got \"{}\"", &text.value),
                            Some(&strong.children[0]),
                        )),
                    },
                    _ => Err(MDError::new(
                        "expected ingrdient ref to be text",
                        Some(&strong.children[0]),
                    )),
                },
                _ => Err(MDError::new("expected single children", Some(node))),
            },
            _ => Err(MDError::new("unsupported element in step", Some(node))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::md_parser::tests::assert_parse;
    use indoc::indoc;

    #[test]
    fn parse_step() {
        let content = indoc! {"
        - Top
            - Nested with *emphasis* and **10 minutes**
            - Nested at the same width
                - Double-nested
        "};
        let mdast = markdown::to_mdast(content, &markdown::ParseOptions::default()).unwrap();
        assert_parse!(Instructions::from_mdast(mdast.children().unwrap()));
    }
}
