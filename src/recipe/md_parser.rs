use markdown::{
    self,
    mdast::Node,
    message::{self, Place},
};
use std::{
    fmt::{self, Display},
    num::ParseFloatError,
};

#[derive(Debug)]
pub struct MDError {
    msg: String,
    place: Option<Place>,
}

impl MDError {
    pub fn new(msg: &str, node: Option<&Node>) -> Self {
        Self {
            msg: msg.to_string(),
            place: node.and_then(|n| {
                n.position()
                    .and_then(|pos| Some(Place::Position(pos.clone())))
            }),
        }
    }
}

pub type MDResult<T> = Result<T, MDError>;

impl From<message::Message> for MDError {
    fn from(value: message::Message) -> Self {
        let msg = format!("{} ({}:{})", value.reason, value.source, value.rule_id);
        Self {
            msg,
            place: value.place.and_then(|p| Some(*p)),
        }
    }
}

impl From<ParseFloatError> for MDError {
    fn from(value: ParseFloatError) -> Self {
        MDError::new(&format!("{}", value), None)
    }
}

impl Display for MDError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)?;
        if let Some(place) = &self.place {
            write!(f, " @ {}", place)?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}

pub struct ASTConsumer<'a> {
    idx: usize,
    nodes: &'a [Node],
}

impl<'a> ASTConsumer<'a> {
    pub fn new(nodes: &'a [Node]) -> Self {
        ASTConsumer { idx: 0, nodes }
    }

    pub fn next(&mut self) -> Result<&'a Node, MDError> {
        if self.idx == self.nodes.len() {
            Err(MDError::new("EOF", None))
        } else {
            let node = &self.nodes[self.idx];
            self.idx += 1;
            Ok(node)
        }
    }

    pub fn consume_to_next_heading(&mut self, depth: u8) -> &[Node] {
        if self.idx == self.nodes.len() {
            &[]
        } else if let Some(elem) = self.nodes[self.idx..].iter().enumerate().find(|&node| {
            if let Node::Heading(heading) = node.1 {
                heading.depth == depth
            } else {
                false
            }
        }) {
            let slice = &self.nodes[self.idx..(self.idx + elem.0)];
            self.idx += elem.0;
            slice
        } else {
            let slice = &self.nodes[self.idx..];
            self.idx = self.nodes.len();
            slice
        }
    }

    pub fn get_remaining(&'a self) -> &'a [Node] {
        if self.idx == self.nodes.len() {
            &[]
        } else {
            &self.nodes[self.idx..]
        }
    }
}

pub fn expect_children(node: &Node, num: usize) -> MDResult<()> {
    match &node.children() {
        Some(children) => {
            if children.len() != num {
                Err(MDError::new(
                    &format!(
                        "expected node to have {} children, but got {}",
                        num,
                        children.len()
                    ),
                    Some(node),
                ))
            } else {
                Ok(())
            }
        }
        None => Err(MDError::new("node cannot have children", Some(node))),
    }
}

pub fn get_heading(node: &Node, depth: u8, name: Option<&str>) -> MDResult<String> {
    // Check that the heading is what we expect.
    if let Node::Heading(heading) = &node {
        // We expect a single Node::Text children at the correct depth.
        if heading.depth != depth {
            Err(MDError::new(
                &format!(
                    "expected heading at depth {}, but got {}",
                    depth, heading.depth
                ),
                Some(node),
            ))
        } else if let Err(e) = expect_children(node, 1) {
            Err(e)
        } else if let Node::Text(text) = &heading.children[0] {
            if let Some(requested_name) = name {
                if text.value != requested_name {
                    Err(MDError::new(
                        &format!(
                            "expected heading \"{}\", but got \"{}\"",
                            requested_name, text.value
                        ),
                        Some(&heading.children[0]),
                    ))
                } else {
                    Ok(text.value.clone())
                }
            } else {
                Ok(text.value.clone())
            }
        } else {
            Err(MDError::new(
                "expected heading to have text child",
                Some(node),
            ))
        }
    } else {
        Err(MDError::new(
            "expected first node to be heading",
            Some(node),
        ))
    }
}

pub fn get_text_from_paragraph<'a>(node: &'a Node) -> MDResult<&'a str> {
    if let Node::Paragraph(para) = &node {
        if let Err(e) = expect_children(node, 1) {
            Err(e)
        } else if let Node::Text(text) = &para.children[0] {
            Ok(&text.value)
        } else {
            Err(MDError::new(
                "expected child to to be text",
                Some(&para.children[0]),
            ))
        }
    } else {
        Err(MDError::new("expected paragraph", Some(node)))
    }
}

pub fn get_parse_options() -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::mdx();
    options.constructs.frontmatter = true;
    options
}
