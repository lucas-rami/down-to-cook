mod ingredients;
mod instructions;
mod md_parser;
mod metadata;
mod unit;

use ingredients::Ingredients;
use instructions::Instructions;
use markdown::{self, mdast::Node};
use md_parser::{get_heading, get_parse_options, ASTConsumer, MDError, MDResult};
use metadata::Metadata;

pub struct Recipe {
    name: String,
    ingredients: Ingredients,
    instructions: Instructions,
    metadata: Metadata,
}

impl Recipe {
    pub fn from_mdast(content: &str) -> MDResult<Self> {
        let md = markdown::to_mdast(content, &get_parse_options())?;
        match md.children() {
            Some(children) => {
                let mut ast_cons = ASTConsumer::new(children);

                // Attempt to parse (optional) metadata and recipe name.
                let first_node = ast_cons.next()?;
                let (metadata, name): (Metadata, String) = match &first_node {
                    Node::Yaml(yaml) => (
                        Metadata::parse(yaml)?,
                        get_heading(ast_cons.next()?, 1, None)?,
                    ),
                    Node::Heading(_) => (Metadata::default(), get_heading(first_node, 1, None)?),
                    _ => Err(MDError::new(
                        "expected YAML frontmatter of heading",
                        Some(first_node),
                    ))?,
                };

                // Attempt to parse "Ingredients" and "Instructions" sections.
                get_heading(ast_cons.next()?, 2, Some("Ingredients"))?;
                let ingredients = Ingredients::parse(ast_cons.consume_to_next_heading(2))?;
                get_heading(ast_cons.next()?, 2, Some("Instructions"))?;
                let instructions = Instructions::parse(ast_cons.consume_to_next_heading(2))?;

                Ok(Self {
                    name,
                    ingredients,
                    instructions,
                    metadata,
                })
            }
            None => Err(MDError::new("empty file", None)),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn parse_recipe() -> MDResult<()> {
        let content = indoc! {"
            # Test recipe
            ## Ingredients

            - Lemons, 1
            - Milk, 50 mL
            - Paprika powder, 1 tbsp (optional, spicy)

            ## Instructions
        "};
        Recipe::from_mdast(content)?;
        Ok(())
    }
}
