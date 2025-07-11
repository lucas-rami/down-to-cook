use std::str::FromStr;
use std::vec;

use super::md_parser::{expect_children, get_heading, get_text_from_paragraph, MDError, MDResult};
use super::unit::Quantity;
use markdown::{self, mdast::Node};

pub enum Ingredients {
    IngredientList(Vec<IngredientOptions>),
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

    fn parse_ingredient_list(node: &Node) -> MDResult<Vec<IngredientOptions>> {
        match node {
            Node::List(list) => Ok(list
                .children
                .iter()
                .map(|n| IngredientOptions::parse(n))
                .collect::<MDResult<Vec<IngredientOptions>>>()?),
            _ => Err(MDError::new("ingredients must be list", Some(node))),
        }
    }
}

pub struct IngredientGroup {
    name: String,
    ingredients: Vec<IngredientOptions>,
}

impl IngredientGroup {
    fn parse(heading: &Node, list: &Node) -> MDResult<Self> {
        Ok(Self {
            name: get_heading(heading, 3, None)?,
            ingredients: Ingredients::parse_ingredient_list(list)?,
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Ingredient {
    name: String,
    quantity: Option<Quantity>,
    alt_quantities: Option<Vec<Quantity>>,
    info: Option<String>,
}

const INFO_FORBIDDEN_CHARS: [char; 3] = ['|', '(', ')'];
const FORBIDDEN_CHARS: [char; 5] = [',', '|', '/', '(', ')'];

impl Ingredient {
    fn from_str(text: &str) -> MDResult<Self> {
        let mut text = text.trim();

        // Determine whether there is additional info between '(' and ')' at the end.
        let info: Option<String> = if text.ends_with(")") {
            Some(
                text.find("(")
                    .ok_or_else(|| MDError::new("found closing parenthesis but no opening", None))
                    .map(|idx| {
                        let info = text[idx + 1..text.len() - 1].trim().to_string();
                        text = &text[..idx];
                        info
                    })?,
            )
        } else {
            None
        };
        if let Some(info_text) = &info {
            if INFO_FORBIDDEN_CHARS.iter().any(|c| info_text.contains(*c)) {
                Err(MDError::new(
                    &format!("additiona info contains forbidden character: {}", info_text),
                    None,
                ))?;
            }
        }

        // Determine whether there is an optional quantity specfied after a ','.
        let (mut quantity, mut alt_quantities): (Option<Quantity>, Option<Vec<Quantity>>) =
            (None, None);
        if let Some(idx) = text.find(",") {
            // We expect at least one quantity, and possibly many alteratives.
            for (i, s) in text[idx + 1..].split('/').enumerate() {
                if FORBIDDEN_CHARS.iter().any(|c| s.contains(*c)) {
                    Err(MDError::new(
                        &format!("quantity contains forbidden character: {}", s),
                        None,
                    ))?;
                }
                let quant = Quantity::from_str(s.trim())
                    .map_err(|e| MDError::new(&format!("failed to parse quantity: {}", e), None))?;
                if i == 0 {
                    quantity = Some(quant);
                } else if i == 1 {
                    alt_quantities = Some(vec![quant]);
                } else {
                    alt_quantities.as_mut().unwrap().push(quant);
                }
            }
            text = &text[..idx];
        }

        let name = text.trim().to_string();
        if name.is_empty() {
            Err(MDError::new(
                &format!("name cannot be empty: {}", name),
                None,
            ))
        } else if FORBIDDEN_CHARS.iter().any(|c| name.contains(*c)) {
            Err(MDError::new("name contains forbidden character", None))
        } else {
            Ok(Self {
                name,
                quantity,
                alt_quantities,
                info,
            })
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct IngredientOptions {
    ingredient: Ingredient,
    alternatives: Option<Vec<Ingredient>>,
}

impl IngredientOptions {
    fn parse(node: &Node) -> MDResult<Self> {
        match node {
            Node::ListItem(item) => expect_children(node, 1)
                .and_then(|_| Self::from_str(get_text_from_paragraph(&item.children[0])?)),
            _ => Err(MDError::new("expected list item", Some(node))),
        }
    }

    fn from_str(text: &str) -> MDResult<Self> {
        let idx = text.find('|').or(Some(text.len())).unwrap();
        let ingredient = Ingredient::from_str(&text[..idx])?;
        let mut alternatives: Vec<Ingredient> = vec![];
        if idx != text.len() {
            for s in text[idx + 1..].split('|') {
                alternatives.push(Ingredient::from_str(s)?);
            }
        }
        Ok(Self {
            ingredient,
            alternatives: (!alternatives.is_empty()).then_some(alternatives),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::unit::{Nominal, Unit, Volume};
    use indoc::indoc;

    // Some quantities
    const ONE_NOMINAL: Quantity = Quantity {
        unit: Unit::Nominal(Nominal),
        amount: 1.,
    };
    const FIFTEEN_ML: Quantity = Quantity {
        unit: Unit::Volume(Volume::Milliliter),
        amount: 15.,
    };
    const THREE_TSP: Quantity = Quantity {
        unit: Unit::Volume(Volume::Teaspoon),
        amount: 3.,
    };
    const ONE_TBSP: Quantity = Quantity {
        unit: Unit::Volume(Volume::Tablespoon),
        amount: 1.,
    };
    const NAME: &str = "name";

    fn simple_ingredient(quantity: Option<&Quantity>, info: Option<&str>) -> Ingredient {
        Ingredient {
            name: NAME.to_string(),
            quantity: quantity.cloned(),
            alt_quantities: None,
            info: info.map(|s| s.to_string()),
        }
    }

    #[test]
    fn parse_ingredient() -> MDResult<()> {
        // Parsing should ignore spaces around key elements.
        let ingr = simple_ingredient(Some(&FIFTEEN_ML), None);
        assert_eq!(Ingredient::from_str("name, 15mL")?, ingr);
        assert_eq!(Ingredient::from_str("name, 15 mL")?, ingr);
        assert_eq!(Ingredient::from_str("   name   ,  15mL  ")?, ingr);

        // "Special units": none, nominal, and custom.
        let one_custom = Quantity {
            unit: Unit::Custom("bunch".to_string()),
            amount: 1.,
        };
        assert_eq!(Ingredient::from_str("name")?, simple_ingredient(None, None));
        assert_eq!(
            Ingredient::from_str("name, 1")?,
            simple_ingredient(Some(&ONE_NOMINAL), None)
        );
        assert_eq!(
            Ingredient::from_str("name, 1 bunch")?,
            simple_ingredient(Some(&one_custom), None)
        );
        // Additional information should still be specifiable when there is no unit.
        assert_eq!(
            Ingredient::from_str("name (info)")?,
            simple_ingredient(None, Some("info"))
        );

        // Additional info (parsing should ignore spaces around and inside paranthesises.
        let ingr_with_info = simple_ingredient(Some(&ONE_TBSP), Some("optional, spicy"));
        assert_eq!(
            Ingredient::from_str("name, 1 tbsp (optional, spicy)")?,
            ingr_with_info
        );
        assert_eq!(
            Ingredient::from_str("name, 1 tbsp(optional, spicy)")?,
            ingr_with_info
        );
        assert_eq!(
            Ingredient::from_str("name, 1 tbsp    (optional, spicy)   ")?,
            ingr_with_info
        );
        assert_eq!(
            Ingredient::from_str("name, 1 tbsp (  optional, spicy )   ")?,
            ingr_with_info
        );

        // Alternative quantities (parsing should ignore spaces around slashes)
        let ingr_with_alts = Ingredient {
            name: NAME.to_string(),
            quantity: Some(FIFTEEN_ML),
            alt_quantities: Some(vec![THREE_TSP, ONE_TBSP]),
            info: None,
        };
        assert_eq!(
            Ingredient::from_str("name, 15mL / 3 tsp / 1tbsp")?,
            ingr_with_alts
        );
        assert_eq!(
            Ingredient::from_str("name, 15mL  /  3 tsp/1tbsp")?,
            ingr_with_alts
        );
        Ok(())
    }

    #[test]
    fn parse_quantity_failure() {
        // Invalid names is not allowed.
        assert!(Ingredient::from_str("").is_err());
        assert!(Ingredient::from_str("  , 15mL").is_err());
        assert!(Ingredient::from_str("na|me, 15mL").is_err());

        // Invalid quantity (and alternatives).
        assert!(Ingredient::from_str("name, a15mL").is_err());
        assert!(Ingredient::from_str("name, 15mL, 15mL").is_err());
        assert!(Ingredient::from_str("name, 15mL / ").is_err());
        assert!(Ingredient::from_str("name, 15mL//3tsp").is_err());

        // Invalid additional information.
        assert!(Ingredient::from_str("name, 15mL (info").is_err());
        assert!(Ingredient::from_str("name, 15mL info)").is_err());
        assert!(Ingredient::from_str("name, 15mL ((info))").is_err());
    }

    #[test]
    fn parse_ingredient_options() -> MDResult<()> {
        let ingr = simple_ingredient(Some(&FIFTEEN_ML), Some("info"));
        let alt1 = simple_ingredient(None, Some("info"));
        let alt2 = simple_ingredient(Some(&ONE_NOMINAL), None);
        let alts = vec![alt1, alt2];

        // No alternatives.
        assert_eq!(
            IngredientOptions::from_str("name, 15ml (info)")?,
            IngredientOptions {
                ingredient: ingr.clone(),
                alternatives: None
            }
        );

        // With alternatives (parsing should ignore spaces around bars).
        assert_eq!(
            IngredientOptions::from_str("name, 15ml (info)|name (info)    |   name, 1")?,
            IngredientOptions {
                ingredient: ingr,
                alternatives: Some(alts.clone())
            }
        );
        Ok(())
    }

    #[test]
    fn parse_ingredient_options_failures() {
        // Invalid single ingredient.
        assert!(IngredientOptions::from_str(", 15ml (info)").is_err());
        // Invalid ingredient with alternatives.
        assert!(IngredientOptions::from_str("name, 15ml (info) | , 15ml (info)").is_err());
        // Missing last alternative.
        assert!(IngredientOptions::from_str("name, 15ml (info) | ").is_err());
    }

    #[test]
    fn parse_ingredient_list() -> MDResult<()> {
        let content = indoc! {"
        - Lemons, 1
        - Milk, 50 mL
        - Paprika powder, 1 tbsp (optional)
        "};
        let mdast = markdown::to_mdast(content, &markdown::ParseOptions::default()).unwrap();
        Ingredients::parse(mdast.children().unwrap())?;
        Ok(())
    }

    #[test]
    fn parse_ingredient_groups() -> MDResult<()> {
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
        Ingredients::parse(mdast.children().unwrap())?;
        Ok(())
    }
}
