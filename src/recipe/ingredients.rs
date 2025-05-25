use std::str::FromStr;

use super::md_parser::{expect_children, get_heading, get_text_from_paragraph, MDError, MDResult};
use super::unit::Quantity;
use markdown::{self, mdast::Node};

pub enum Ingredients {
    IngredientList(Vec<Ingredient>),
    IngredientGroups(Vec<IngredientGroup>),
}

impl Ingredients {
    pub fn parse(nodes: &[Node]) -> MDResult<Self> {
        match nodes.len() {
            0 => Ok(Self::IngredientList(vec![])),
            1 => Ok(Self::IngredientList(Self::parse_ingredient_list(
                &nodes[0],
            )?)),
            _ => {
                // We expect sequences of the following form:
                // - heading at depth 3 (defining the ingredient group's name)
                // - list of ingredients
                Ok(Self::IngredientGroups(
                    nodes
                        .chunks(2)
                        .map(|group| -> MDResult<IngredientGroup> {
                            if group.len() == 1 {
                                Err(MDError::new("malformed ingredient group", Some(&group[0])))
                            } else {
                                IngredientGroup::parse(&group[0], &group[1])
                            }
                        })
                        .collect::<MDResult<Vec<IngredientGroup>>>()?,
                ))
            }
        }
    }

    fn parse_ingredient_list(node: &Node) -> MDResult<Vec<Ingredient>> {
        match node {
            Node::List(list) => Ok(list
                .children
                .iter()
                .map(|n| Ingredient::parse(n))
                .collect::<MDResult<Vec<Ingredient>>>()?),
            _ => Err(MDError::new("ingredients must be list", Some(node))),
        }
    }
}

pub struct IngredientGroup {
    name: String,
    ingredients: Vec<Ingredient>,
}

impl IngredientGroup {
    fn parse(heading: &Node, list: &Node) -> MDResult<Self> {
        Ok(Self {
            name: get_heading(heading, 3, None)?,
            ingredients: Ingredients::parse_ingredient_list(list)?,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct Ingredient {
    name: String,
    quantity: Option<Quantity>,
    attributes: Vec<String>,
}

impl Ingredient {
    fn parse(node: &Node) -> MDResult<Self> {
        match node {
            Node::ListItem(item) => expect_children(node, 1)
                .and_then(|_| Self::from_str(get_text_from_paragraph(&item.children[0])?)),
            _ => Err(MDError::new("expected list item", Some(node))),
        }
    }

    fn from_str(text: &str) -> MDResult<Self> {
        let components: Vec<&str> = text.split(',').map(|s| s.trim()).collect();
        if components.len() < 2 {
            return Err(MDError::new(
                "ingredient must be formatted as <name>, <quantity> [, <leftover>]*",
                None,
            ));
        }

        Ok(Self {
            name: components[0].to_string(),
            quantity: Quantity::from_str(components[1]).map_or(None, |q| Some(q)),
            attributes: components[2..].iter().map(|s| s.to_string()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::{
        md_parser::tests::{assert_parse, assert_parse_eq},
        unit::{Nominal, Unit, Volume},
    };
    use indoc::indoc;

    fn get_ingredient(name: &str, quantity: Option<(Unit, f32)>, leftover: &[&str]) -> Ingredient {
        Ingredient {
            name: name.to_string(),
            quantity: quantity.map(|(unit, amount)| Quantity { unit, amount }),
            attributes: leftover.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn parse_ingredient() {
        let nominal = Unit::Nominal(Nominal);
        assert_parse_eq!(
            Ingredient::from_str("Lemons, 1"),
            get_ingredient("Lemons", Some((nominal, 1.)), &[])
        );

        let ml = Unit::Volume(Volume::Milliliter);
        assert_parse_eq!(
            Ingredient::from_str("Milk, 50 mL"),
            get_ingredient("Milk", Some((ml.clone(), 50.)), &[])
        );
        assert_parse_eq!(
            Ingredient::from_str("   Milk   ,  50mL  "),
            get_ingredient("Milk", Some((ml, 50.)), &[])
        );

        let custom = Unit::Custom("bunch".to_string());
        assert_parse_eq!(
            Ingredient::from_str("Basil, 1 bunch"),
            get_ingredient("Basil", Some((custom, 1.)), &[])
        );

        let tbsp = Unit::Volume(Volume::Tablespoon);
        assert_parse_eq!(
            Ingredient::from_str("Paprika powder, 1 tbsp, optional, [spicy]"),
            get_ingredient("Paprika powder", Some((tbsp, 1.)), &["optional", "[spicy]"])
        );
    }

    #[test]
    fn parse_ingredient_list() {
        let content = indoc! {"
        - Lemons, 1
        - Milk, 50 mL
        - Paprika powder, 1 tbsp, optional
        "};
        let mdast = markdown::to_mdast(content, &markdown::ParseOptions::default()).unwrap();
        assert_parse!(Ingredients::parse(mdast.children().unwrap()));
    }

    #[test]
    fn parse_ingredient_groups() {
        let content = indoc! {"
        ### Group 1
        - Thing 1, 1
        - Thing 2, 1
        ### Group 2
        - Thing 3, 1
        ### Group 3
        - Thing 4, 1
        - Thing 5, 1
        - Thing 6, 1
        "};
        let mdast = markdown::to_mdast(content, &markdown::ParseOptions::default()).unwrap();
        assert_parse!(Ingredients::parse(mdast.children().unwrap()));
    }
}
