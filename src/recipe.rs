mod ingredient;
mod md_parser;
mod step;
mod unit;

use ingredient::IngredientList;
use markdown::{self};
use md_parser::{get_heading, ASTConsumer, MDError};
use step::Steps;

pub struct Recipe {
    name: String,
    ingredients: IngredientList,
    steps: Steps,
}

impl Recipe {
    pub fn from_mdast(content: &str) -> Result<Self, MDError> {
        let md = markdown::to_mdast(content, &markdown::ParseOptions::default())?;
        match md.children() {
            Some(children) => {
                let mut ast_cons = ASTConsumer::new(children);
                let name = get_heading(ast_cons.consume_next()?, 1, None)?;
                get_heading(ast_cons.consume_next()?, 2, Some("Ingredients"))?;
                let ingredients = IngredientList::from_mdast(ast_cons.consume_to_next_heading(2))?;
                get_heading(ast_cons.consume_next()?, 2, Some("Steps"))?;
                let steps = Steps::from_mdast(ast_cons.consume_to_next_heading(2))?;
                Ok(Self {
                    name,
                    ingredients,
                    steps,
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
            - Paprika powder, 1 tbsp, optional, [spicy]

            ## Steps
        "};
        print!(
            "{:#?}",
            markdown::to_mdast(content, &markdown::ParseOptions::default()).unwrap()
        );
        assert_parse!(Recipe::from_mdast(content));
    }
}
