use super::unit::Unit;
use crate::recipe::{
    md_parser::{MDError, MDResult},
    unit::{Distance, Nominal, Quantity, QuantityOf},
};
use markdown::mdast::Yaml;
use saphyr::LoadableYamlNode;
use std::{collections::HashMap, str::FromStr};

pub struct Metadata {
    tags: Vec<String>,
    quantity: Quantity,
    sizes: HashMap<String, SizeInfo>,
    others: HashMap<String, String>,
}

const TAGS: &str = "tags";
const QUANTITY: &str = "quantity";
const SIZE_PREFIX: &str = "size | ";

impl Metadata {
    pub fn parse(yaml: &Yaml) -> MDResult<Self> {
        let metadata =
            saphyr::Yaml::load_from_str(&yaml.value).map_err(|e| MDError::new(e.info(), None))?;
        let mapping = (metadata.len() == 1)
            .then(|| &metadata[0])
            .ok_or(MDError::new(
                "expected single YAML document in frontmatter",
                None,
            ))?
            .as_mapping()
            .ok_or(MDError::new(
                "expected top-level element to be mapping",
                None,
            ))?;

        let mut this = Self::default();

        for (key, value) in mapping {
            let key: &str = key
                .as_str()
                .ok_or(MDError::new("expected string key", None))?;
            match key {
                TAGS => Self::parse_tags(value, &mut this.tags)?,
                QUANTITY => Self::parse_quantity(value, &mut this.quantity)?,
                _ => {
                    if key.starts_with(SIZE_PREFIX) {
                        Self::parse_size(&key[SIZE_PREFIX.len()..], value, &mut this.sizes)?;
                    } else {
                        Self::parse_others(&key, value, &mut this.others)?;
                    }
                }
            }
        }

        Ok(this)
    }

    fn get_tag(tag: &str) -> MDResult<&str> {
        if !tag.starts_with("#") {
            return Err(MDError::new(
                &format!("tag must start with '#' character"),
                None,
            ));
        }
        let no_hash = &tag["#".len()..];
        if no_hash
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '/' && c != '-' && c != '_')
        {
            return Err(MDError::new(
                &format!("tag {:?} contains forbidden characters", tag),
                None,
            ));
        }
        Ok(no_hash)
    }

    fn parse_tags(value: &saphyr::Yaml<'_>, tags: &mut Vec<String>) -> MDResult<()> {
        let value = value.as_sequence().ok_or(MDError::new(
            &format!("expected sequence under {:?}", TAGS),
            None,
        ))?;
        for tag in value {
            let s_tag = tag
                .as_str()
                .ok_or(MDError::new("expected string tag", None))?;
            Self::get_tag(s_tag).inspect(|t| tags.push(t.to_string()))?;
        }
        Ok(())
    }

    fn parse_quantity(value: &saphyr::Yaml<'_>, quantity: &mut Quantity) -> MDResult<()> {
        let value = value.as_str().ok_or(MDError::new(
            &format!("expected string under {:?}", QUANTITY),
            None,
        ))?;
        *quantity = Quantity::from_str(value)?;
        Ok(())
    }

    fn parse_size(
        key: &str,
        value: &saphyr::Yaml<'_>,
        sizes: &mut HashMap<String, SizeInfo>,
    ) -> MDResult<()> {
        if key.is_empty() {
            return Err(MDError::new("sized object must have a name", None));
        }
        let size = value.as_str().ok_or(MDError::new(
            &format!("expected string size attribute for {:?}", key),
            None,
        ))?;
        sizes.insert(key.to_string(), SizeInfo::from_str(size)?);
        Ok(())
    }

    fn parse_others(
        key: &str,
        value: &saphyr::Yaml<'_>,
        others: &mut HashMap<String, String>,
    ) -> MDResult<()> {
        let value = value.as_str().ok_or(MDError::new(
            "for unknown keys, only string values are supported",
            None,
        ))?;
        if let Some(_) = others.insert(key.to_string(), value.to_string()) {
            return Err(MDError::new(
                &format!("duplicate metadata key {:?}", key),
                None,
            ));
        }
        Ok(())
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            tags: vec![],
            quantity: Quantity::new(&Unit::Nominal(Nominal {}), 1.),
            sizes: HashMap::new(),
            others: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SizeInfo {
    quantity: QuantityOf<Distance>,
    unit_mod: Option<UnitMod>,
}

impl FromStr for SizeInfo {
    type Err = MDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut info_s: &str = s.trim();
        let unit_mod = info_s.ends_with("°").then(|| {
            info_s = &info_s[..info_s.len() - "°".len()];
            UnitMod::RadialDistance
        });
        Ok(Self {
            quantity: QuantityOf::from_str(info_s)
                .map_err(|e| MDError::new(&format!("failed to parse quantity: {}", e), None))?,
            unit_mod,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnitMod {
    RadialDistance,
}

#[cfg(test)]
pub mod tests {
    use indoc::indoc;
    use markdown::mdast::Node;
    use saphyr::LoadableYamlNode;
    use std::collections::HashMap;

    use crate::recipe::{
        md_parser::{get_parse_options, MDResult},
        metadata::{SizeInfo, UnitMod},
        unit::{Distance, Quantity, QuantityOf, Unit, Volume},
    };

    use super::Metadata;

    fn to_yaml(s: &str) -> saphyr::Yaml<'_> {
        let metadata = saphyr::Yaml::load_from_str(s).unwrap();
        return metadata[0].clone();
    }

    #[test]
    fn parse_tags() -> MDResult<()> {
        // Basic case.
        let mut tags: Vec<String> = vec![];
        Metadata::parse_tags(&to_yaml("- \"#tag1\"\n- \"#tag2\"\n- \"#tag3\""), &mut tags)?;
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);

        // There is no de-duplication.
        tags.clear();
        Metadata::parse_tags(&to_yaml("- \"#tag\"\n- \"#tag\""), &mut tags)?;
        assert_eq!(tags, vec!["tag", "tag"]);

        // Special characters and random spaces.
        tags.clear();
        Metadata::parse_tags(&to_yaml("-   \"#t/a/g\"  \n- \"#t-a_g\" "), &mut tags)?;
        assert_eq!(tags, vec!["t/a/g", "t-a_g"]);
        Ok(())
    }

    #[test]
    fn parse_tags_failure() {
        // Missing #.
        assert!(Metadata::get_tag("- \"tag\"").is_err());
        // Double #.
        assert!(Metadata::get_tag("- \"##tag\"").is_err());
        // Invalid character.
        assert!(Metadata::get_tag("- \"#tag.1\"").is_err());
    }

    #[test]
    fn parse_size() -> MDResult<()> {
        let mut sizes: HashMap<String, SizeInfo> = HashMap::new();
        let ten_cm = SizeInfo {
            quantity: QuantityOf {
                unit: Distance::Centimeter,
                amount: 10.,
            },
            unit_mod: None,
        };

        // Basic case.
        Metadata::parse_size("pan", &to_yaml("10cm"), &mut sizes)?;
        assert_eq!(*sizes.get("pan").unwrap(), ten_cm);

        let ten_radial_cm = {
            let mut tmp = ten_cm.clone();
            tmp.unit_mod = Some(UnitMod::RadialDistance);
            tmp
        };

        // With radial distance modifier.
        sizes.clear();
        Metadata::parse_size("pan", &to_yaml("10cm°"), &mut sizes)?;
        assert_eq!(*sizes.get("pan").unwrap(), ten_radial_cm);

        // Spaces around modfier do not matter.
        sizes.clear();
        Metadata::parse_size("pan", &to_yaml("10cm  °   "), &mut sizes)?;
        assert_eq!(*sizes.get("pan").unwrap(), ten_radial_cm);

        Ok(())
    }

    #[test]
    fn parse_size_failures() {
        // Only distance-typed units are supported.
        assert!(Metadata::parse_size("pan", &to_yaml("10mL°"), &mut HashMap::new()).is_err());
    }

    #[test]
    fn parse_others() -> MDResult<()> {
        let mut others: HashMap<String, String> = HashMap::new();

        // Basic case.
        Metadata::parse_others("key", &to_yaml("value"), &mut others)?;
        assert_eq!(*others.get("key").unwrap(), "value");

        Ok(())
    }

    #[test]
    fn parse_others_failures() {
        // The value must be a string.
        assert!(
            Metadata::parse_others("key", &to_yaml("- value1\n- value2"), &mut HashMap::new())
                .is_err()
        );
    }

    #[test]
    fn parse_metadata() -> MDResult<()> {
        let content = indoc! {"
            ---
            tags:
              - \"#tag1\"
              - \"#tag2\"
            quantity: 150ml
            size | pan: 10cm
            size | whatever: 10cm
            random: something
            ---
        "};
        let md = markdown::to_mdast(content, &get_parse_options())?;
        if let Node::Yaml(yaml) = &md.children().unwrap()[0] {
            let meta = Metadata::parse(yaml)?;
            assert_eq!(meta.tags, vec!["tag1", "tag2"]);
            assert_eq!(
                meta.quantity,
                Quantity {
                    unit: Unit::Volume(Volume::Milliliter),
                    amount: 150.
                }
            );
            let size = SizeInfo {
                quantity: QuantityOf {
                    unit: Distance::Centimeter,
                    amount: 10.,
                },
                unit_mod: None,
            };
            assert_eq!(*meta.sizes.get("pan").unwrap(), size);
            assert_eq!(*meta.sizes.get("whatever").unwrap(), size);
            assert_eq!(*meta.others.get("random").unwrap(), "something");
        } else {
            panic!("should be YAML!");
        }
        Ok(())
    }
}
