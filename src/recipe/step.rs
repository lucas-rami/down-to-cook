use super::{
    md_parser::{parse_quantity, MDError},
    unit::Unit,
};
use markdown::mdast::{ListItem, Node};

#[derive(Clone, PartialEq)]
pub struct Steps {
    steps: Vec<Step>,
}

impl Steps {
    pub fn from_mdast(nodes: &[Node]) -> Result<Self, MDError> {
        match nodes.len() {
            0 => Ok(Self { steps: vec![] }),
            1 => match &nodes[0] {
                Node::List(list) => Ok(Self {
                    steps: list
                        .children
                        .iter()
                        .map(|n| -> Result<Step, MDError> {
                            match n {
                                Node::ListItem(item) => Step::from_list_item(item),
                                _ => Err(MDError::new("expected list item", Some(&n))),
                            }
                        })
                        .collect::<Result<Vec<Step>, _>>()?,
                }),
                _ => Err(MDError::new("steps must be list", Some(&nodes[0]))),
            },
            _ => Err(MDError::new("expected single list node for steps", None)),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum TextElem {
    Text(String),
    IngredientRef(String),
    Timer(f32, Unit),
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
                    Node::Text(text) => {
                        let (quantity, unit) = parse_quantity(&text.value, false)?;
                        let unit = unit.unwrap();
                        if unit.is_time() {
                            Ok(Self::Timer(quantity, unit))
                        } else {
                            Err(MDError::new(
                                &format!("expected time unit but unit is {:?}", unit),
                                Some(&strong.children[0]),
                            ))
                        }
                    }
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

#[derive(Clone, PartialEq)]
pub struct Step {
    description: Vec<TextElem>,
    substeps: Steps,
}

impl Step {
    fn from_list_item(item: &ListItem) -> Result<Self, MDError> {
        match item.children.len() {
            0 => Ok(Self {
                description: vec![],
                substeps: Steps::from_mdast(&[])?,
            }),
            _ => match &item.children[0] {
                Node::Paragraph(para) => Ok(Self {
                    description: Self::parse_description(&para.children)?,
                    substeps: Steps::from_mdast(&item.children[1..])?,
                }),
                _ => Err(MDError::new("expected paragraph", Some(&item.children[0]))),
            },
        }
    }

    fn parse_description(nodes: &[Node]) -> Result<Vec<TextElem>, MDError> {
        Ok(nodes
            .iter()
            .map(|n| TextElem::from_node(n))
            .collect::<Result<Vec<TextElem>, _>>()?)
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
        "};
        let mdast = markdown::to_mdast(content, &markdown::ParseOptions::default()).unwrap();
        assert_parse!(Steps::from_mdast(mdast.children().unwrap()));
    }
}
