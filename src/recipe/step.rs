use super::md_parser::MDError;
use markdown::mdast::Node;

pub struct Steps {}

impl Steps {
    pub fn from_mdast(_nodes: &[Node]) -> Result<Self, MDError> {
        Ok(Steps {})
    }
}

pub struct Step {}

impl Step {
    pub fn from_mdast(_nodes: &[Node]) -> Result<Self, MDError> {
        todo!()
    }
}
