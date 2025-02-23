use markdown::{self, mdast::Node};

use crate::md_parser::{
    expect_children, get_heading, get_text_from_paragraph, ASTConsumer, MDError,
};

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

#[derive(PartialEq, Clone, Debug)]
pub enum Unit {
    // Mass
    Gram,
    Kilogram,
    Ounce,
    Pound,
    // Volume
    Milliliter,
    Centiliter,
    Liter,
    Teaspoon,
    Tablespoon,
    FluidOunce,
    Cup,
    Gallon,
    // Distance,
    Millimeter,
    Centimeter,
    Inches,
    // Temperature
    Celsius,
    Farenheit,
    // Unknown
    Custom(String),
}

impl Unit {
    pub fn decode(text: &str) -> Self {
        match &text.to_lowercase()[..] {
            "g" => Self::Gram,
            "kg" => Self::Kilogram,
            "oz" => Self::Ounce,
            "lbs" => Self::Pound,
            "ml" => Self::Milliliter,
            "cl" => Self::Centiliter,
            "l" => Self::Liter,
            "tsp" => Self::Teaspoon,
            "tbsp" => Self::Tablespoon,
            "fl oz" | "fl. oz." => Self::FluidOunce,
            "cup" => Self::Cup,
            "gal" => Self::Liter,
            "°c" => Self::Celsius,
            "°f" => Self::Farenheit,
            "mm" => Self::Millimeter,
            "cm" => Self::Centimeter,
            "in" => Self::Inches,
            _ => Self::Custom(text.to_string()),
        }
    }

    pub fn sanitize(&self, quantity: f32) -> (Self, f32) {
        match *self {
            // Sanitize
            Self::Ounce => (Self::Gram, 28. * quantity),
            Self::Pound => (Self::Gram, 450. * quantity),
            Self::Teaspoon => (Self::Milliliter, 5. * quantity),
            Self::Tablespoon => (Self::Milliliter, 15. * quantity),
            Self::Cup => (Self::Milliliter, 240. * quantity),
            Self::FluidOunce => (Self::Milliliter, 29. * quantity), // halfway between US and UK conventions
            Self::Gallon => (Self::Liter, 3.785 * quantity),
            Self::Farenheit => (Self::Celsius, (quantity - 32.) * 5. / 9.),
            Self::Inches => (Self::Centimeter, 2.5 * quantity),
            // Already sanitized
            _ => (self.clone(), quantity),
        }
    }
}

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

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    macro_rules! assert_parse {
        ( $res:expr ) => {
            assert!($res.inspect_err(|e| print!("{}", e)).is_ok());
        };
    }

    macro_rules! assert_parse_eq {
        ( $res:expr, $val:expr ) => {
            assert!($res.inspect_err(|e| print!("{}", e)).is_ok());
            assert_eq!($res.unwrap(), $val);
        };
    }

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
        - Paprika powder, 1 tbsp, optional, [spicy]
        "};
        let mdast = markdown::to_mdast(content, &markdown::ParseOptions::default()).unwrap();
        assert_parse!(IngredientList::from_mdast(mdast.children().unwrap()));
    }
}
