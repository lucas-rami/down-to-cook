use super::md_parser::{expect_children, get_text_from_paragraph, MDError};
use super::unit::Unit;
use markdown::{self, mdast::Node};

pub struct IngredientList {
    ingredients: Vec<Ingredient>,
}

impl IngredientList {
    pub fn from_mdast(nodes: &[Node]) -> Result<Self, MDError> {
        // We simply expect a single node that is a list at this point
        if nodes.is_empty() {
            return Ok(Self {
                ingredients: vec![],
            });
        }
        if nodes.len() != 1 {
            return Err(MDError::new(
                "expect single list node for ingredients",
                None,
            ));
        }
        if let Node::List(list) = &nodes[0] {
            Ok(Self {
                ingredients: list
                    .children
                    .iter()
                    .map(|n| -> Result<Ingredient, MDError> {
                        if let Node::ListItem(item) = n {
                            if let Err(e) = expect_children(&n, 1) {
                                Err(e)
                            } else {
                                Ingredient::from_str(get_text_from_paragraph(&item.children[0])?)
                            }
                        } else {
                            Err(MDError::new("expected list item", Some(&n)))
                        }
                    })
                    .collect::<Result<Vec<Ingredient>, _>>()?,
            })
        } else {
            Err(MDError::new("ingredients must be list", Some(&nodes[0])))
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
    pub fn new(name: &str, quantity: f32, unit: Option<Unit>, leftover: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            quantity,
            unit,
            attributes: leftover.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn from_str(text: &str) -> Result<Self, MDError> {
        let components: Vec<&str> = text.split(',').map(|s| s.trim()).collect();
        if components.len() < 2 {
            return Err(MDError::new(
                "ingredient must be formatted as <name>, <quantity> [, <leftover>]*",
                None,
            ));
        }

        let (quantity, unit) = if let Some(idx) = components[1].find(|c: char| c.is_alphabetic()) {
            let (quantity, unit) = components[1].split_at(idx);
            (
                quantity.trim().parse::<f32>()?,
                Some(Unit::decode(unit.trim())),
            )
        } else {
            (components[1].to_string().parse::<f32>()?, None)
        };

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

    #[test]
    fn parse_ingredient() {
        assert_parse_eq!(
            Ingredient::from_str("Lemons, 1"),
            Ingredient::new("Lemons", 1., None, &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("Milk, 50 mL"),
            Ingredient::new("Milk", 50., Some(Unit::Milliliter), &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("   Milk   ,  50mL  "),
            Ingredient::new("Milk", 50., Some(Unit::Milliliter), &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("Basil, 1 bunch"),
            Ingredient::new("Basil", 1., Some(Unit::Custom("bunch".to_string())), &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("Paprika powder, 1 tbsp, optional, [spicy]"),
            Ingredient::new(
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
