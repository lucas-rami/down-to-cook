use super::md_parser::{expect_children, get_text_from_paragraph, parse_quantity, MDError};
use super::unit::Unit;
use markdown::{self, mdast::Node};

pub struct IngredientList {
    ingredients: Vec<Ingredient>,
}

impl IngredientList {
    pub fn from_mdast(nodes: &[Node]) -> Result<Self, MDError> {
        match nodes.len() {
            0 => Ok(Self {
                ingredients: vec![],
            }),
            1 => match &nodes[0] {
                Node::List(list) => Ok(Self {
                    ingredients: list
                        .children
                        .iter()
                        .map(|n| -> Result<Ingredient, MDError> {
                            match n {
                                Node::ListItem(item) => expect_children(&n, 1).and_then(|_| {
                                    Ingredient::from_str(get_text_from_paragraph(
                                        &item.children[0],
                                    )?)
                                }),
                                _ => Err(MDError::new("expected list item", Some(&n))),
                            }
                        })
                        .collect::<Result<Vec<Ingredient>, _>>()?,
                }),
                _ => Err(MDError::new("ingredients must be list", Some(&nodes[0]))),
            },
            _ => Err(MDError::new(
                "expected single list node for ingredients",
                None,
            )),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Ingredient {
    name: String,
    quantity: f32,
    unit: Option<Unit>,
    attributes: Vec<String>,
}

impl Ingredient {
    pub fn from_str(text: &str) -> Result<Self, MDError> {
        let components: Vec<&str> = text.split(',').map(|s| s.trim()).collect();
        if components.len() < 2 {
            return Err(MDError::new(
                "ingredient must be formatted as <name>, <quantity> [, <leftover>]*",
                None,
            ));
        }

        let (quantity, unit) = parse_quantity(components[1], true)?;
        Ok(Self {
            name: components[0].to_string(),
            quantity,
            unit,
            attributes: components[2..].iter().map(|s| s.to_string()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::md_parser::tests::{assert_parse, assert_parse_eq};
    use indoc::indoc;

    fn get_ingredient(
        name: &str,
        quantity: f32,
        unit: Option<Unit>,
        leftover: &[&str],
    ) -> Ingredient {
        Ingredient {
            name: name.to_string(),
            quantity,
            unit,
            attributes: leftover.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn parse_ingredient() {
        assert_parse_eq!(
            Ingredient::from_str("Lemons, 1"),
            get_ingredient("Lemons", 1., None, &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("Milk, 50 mL"),
            get_ingredient("Milk", 50., Some(Unit::Milliliter), &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("   Milk   ,  50mL  "),
            get_ingredient("Milk", 50., Some(Unit::Milliliter), &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("Basil, 1 bunch"),
            get_ingredient("Basil", 1., Some(Unit::Custom("bunch".to_string())), &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("Paprika powder, 1 tbsp, optional, [spicy]"),
            get_ingredient(
                "Paprika powder",
                1.,
                Some(Unit::Tablespoon),
                &["optional", "[spicy]"]
            )
        );
    }

    #[test]
    fn parse_ingredients_list() {
        let content = indoc! {"
        - Lemons, 1
        - Milk, 50 mL
        - Paprika powder, 1 tbsp, optional
        "};
        let mdast = markdown::to_mdast(content, &markdown::ParseOptions::default()).unwrap();
        assert_parse!(IngredientList::from_mdast(mdast.children().unwrap()));
    }
}
