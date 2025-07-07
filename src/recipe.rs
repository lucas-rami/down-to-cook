mod ingredients;
mod instructions;
mod md_parser;
mod unit;

use ingredients::Ingredients;
use instructions::Instructions;
use markdown::{self};
use md_parser::{get_heading, ASTConsumer, MDError};

pub struct Recipe {
    name: String,
    ingredients: Ingredients,
    instructions: Instructions,
}

impl Recipe {
    pub fn from_mdast(content: &str) -> Result<Self, MDError> {
        let md = markdown::to_mdast(content, &markdown::ParseOptions::default())?;
        match md.children() {
            Some(children) => {
                let mut ast_cons = ASTConsumer::new(children);
                let name = get_heading(ast_cons.next()?, 1, None)?;
                get_heading(ast_cons.next()?, 2, Some("Ingredients"))?;
                let ingredients = Ingredients::parse(ast_cons.consume_to_next_heading(2))?;
                get_heading(ast_cons.next()?, 2, Some("Instructions"))?;
                let instructions = Instructions::parse(ast_cons.consume_to_next_heading(2))?;
                Ok(Self {
                    name,
                    ingredients,
                    instructions,
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
    use md_parser::tests::assert_parse;

    #[test]
    fn parse_recipe() {
        let content = indoc! {"
            # Test recipe
            ## Ingredients

            - Lemons, 1
            - Milk, 50 mL
            - Paprika powder, 1 tbsp (optional, spicy)

            ## Instructions
        "};
        assert_parse!(Recipe::from_mdast(content));
    }
}
