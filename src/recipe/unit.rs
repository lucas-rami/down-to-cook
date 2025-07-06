use std::{error, fmt, num::ParseFloatError, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub enum Unit {
    Nominal(Nominal),
    Mass(Mass),
    Volume(Volume),
    Distance(Distance),
    Temperature(Temperature),
    Time(Time),
    Custom(String),
}

impl FromStr for Unit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(unit) = Nominal::from_str(s) {
            Ok(Self::Nominal(unit))
        } else if let Ok(unit) = Mass::from_str(s) {
            Ok(Self::Mass(unit))
        } else if let Ok(unit) = Volume::from_str(s) {
            Ok(Self::Volume(unit))
        } else if let Ok(unit) = Distance::from_str(s) {
            Ok(Self::Distance(unit))
        } else if let Ok(unit) = Temperature::from_str(s) {
            Ok(Self::Temperature(unit))
        } else if let Ok(unit) = Time::from_str(s) {
            Ok(Self::Time(unit))
        } else {
            Err(())
        }
    }
}

impl From<&str> for Unit {
    fn from(value: &str) -> Self {
        match Self::from_str(value) {
            Ok(unit) => unit,
            Err(_) => Self::Custom(value.to_string()),
        }
    }
}

pub trait UnitTrait<'a>: Clone + FromStr<Err = ()> {
    fn sanitize(&self, quantity: f32) -> (Self, f32) {
        (self.clone(), quantity)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Nominal;

impl FromStr for Nominal {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self)
        } else {
            Err(())
        }
    }
}

impl UnitTrait<'_> for Nominal {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mass {
    Gram,
    Kilogram,
    Ounce,
    Pound,
}

impl FromStr for Mass {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "g" => Ok(Self::Gram),
            "kg" => Ok(Self::Kilogram),
            "oz" => Ok(Self::Ounce),
            "lbs" => Ok(Self::Pound),
            _ => Err(()),
        }
    }
}

impl UnitTrait<'_> for Mass {
    fn sanitize(&self, quantity: f32) -> (Self, f32) {
        match self {
            Self::Ounce => (Self::Gram, 28. * quantity),
            Self::Pound => (Self::Gram, 450. * quantity),
            _ => (*self, quantity),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Volume {
    Milliliter,
    Centiliter,
    Liter,
    Teaspoon,
    Tablespoon,
    FluidOunce,
    Cup,
    Gallon,
}

impl FromStr for Volume {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "ml" => Ok(Self::Milliliter),
            "cl" => Ok(Self::Centiliter),
            "l" => Ok(Self::Liter),
            "tsp" => Ok(Self::Teaspoon),
            "tbsp" => Ok(Self::Tablespoon),
            "fl oz" | "fl. oz." => Ok(Self::FluidOunce),
            "cup" => Ok(Self::Cup),
            "gal" => Ok(Self::Gallon),
            _ => Err(()),
        }
    }
}

impl UnitTrait<'_> for Volume {
    fn sanitize(&self, quantity: f32) -> (Self, f32) {
        match self {
            Self::Teaspoon => (Self::Milliliter, 5. * quantity),
            Self::Tablespoon => (Self::Milliliter, 15. * quantity),
            Self::Cup => (Self::Milliliter, 240. * quantity),
            // Halfway between US and UK conventions; for more precision, use a better unit.
            Self::FluidOunce => (Self::Milliliter, 29. * quantity),
            Self::Gallon => (Self::Liter, 3.785 * quantity),
            _ => (*self, quantity),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Distance {
    Millimeter,
    Centimeter,
    Inches,
}

impl FromStr for Distance {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "mm" => Ok(Self::Millimeter),
            "cm" => Ok(Self::Centimeter),
            "in" => Ok(Self::Inches),
            _ => Err(()),
        }
    }
}

impl UnitTrait<'_> for Distance {
    fn sanitize(&self, quantity: f32) -> (Self, f32) {
        match self {
            Self::Inches => (Self::Centimeter, 2.5 * quantity),
            _ => (*self, quantity),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Temperature {
    Celsius,
    Farenheit,
}

impl FromStr for Temperature {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "°c" | "c" => Ok(Self::Celsius),
            "°f" | "f" => Ok(Self::Farenheit),
            _ => Err(()),
        }
    }
}

impl UnitTrait<'_> for Temperature {
    fn sanitize(&self, quantity: f32) -> (Self, f32) {
        match self {
            Self::Farenheit => (Self::Celsius, (quantity - 32.) * 5. / 9.),
            _ => (*self, quantity),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Time {
    Second,
    Minute,
    Hour,
}

impl FromStr for Time {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "s" | "sec" | "sec." | "second" | "seconds" => Ok(Self::Second),
            "min" | "min." | "minute" | "minutes" => Ok(Self::Minute),
            "h" | "hour" | "hours" => Ok(Self::Hour),
            _ => Err(()),
        }
    }
}

impl UnitTrait<'_> for Time {}

fn f_split_quantity(c: char) -> bool {
    c.is_alphabetic() || c == '°'
}

#[derive(Clone, Debug, PartialEq)]
pub struct Quantity {
    pub unit: Unit,
    pub amount: f32,
}

impl FromStr for Quantity {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.find(f_split_quantity) {
            Some(idx) => {
                let (quantity, unit) = s.split_at(idx);
                Ok(Self {
                    unit: Unit::from(unit.trim()),
                    amount: quantity.trim().parse::<f32>()?,
                })
            }
            None => Ok(Self {
                unit: Unit::Nominal(Nominal),
                amount: s.trim().parse::<f32>()?,
            }),
        }
    }
}

// A Quantity can always be derived from a QuantityOf<T>.
macro_rules! from_quantity_of {
    ( $unit_enum:expr, $unit_ty:ty ) => {
        impl From<QuantityOf<$unit_ty>> for Quantity {
            fn from(value: QuantityOf<$unit_ty>) -> Self {
                Self {
                    unit: $unit_enum(value.unit),
                    amount: value.amount,
                }
            }
        }
    };
}
from_quantity_of!(Unit::Nominal, Nominal);
from_quantity_of!(Unit::Mass, Mass);
from_quantity_of!(Unit::Volume, Volume);
from_quantity_of!(Unit::Distance, Distance);
from_quantity_of!(Unit::Temperature, Temperature);
from_quantity_of!(Unit::Time, Time);

#[derive(Clone, Debug, PartialEq)]
pub enum ParseQuantityOfError {
    InvalidUnit(String),
    InvalidAmount(String, ParseFloatError),
}

impl fmt::Display for ParseQuantityOfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUnit(s) => write!(f, "unknown unit \"{}\"", s),
            Self::InvalidAmount(s, f_err) => {
                write!(f, "could not parse amount \"{}\": {}", s, f_err)
            }
        }
    }
}

impl error::Error for ParseQuantityOfError {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct QuantityOf<T: for<'a> UnitTrait<'a>> {
    pub unit: T,
    pub amount: f32,
}

impl<T> FromStr for QuantityOf<T>
where
    T: for<'a> UnitTrait<'a>,
{
    type Err = ParseQuantityOfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_at = s.find(f_split_quantity).map_or(s.len(), |s| s);
        let (quantity, unit) = s.split_at(split_at);
        let quantity = quantity.trim();
        let unit = unit.trim();
        Ok(Self {
            unit: T::from_str(unit)
                .map_err(|_| ParseQuantityOfError::InvalidUnit(unit.to_string()))?,
            amount: quantity
                .parse::<f32>()
                .map_err(|e| ParseQuantityOfError::InvalidAmount(quantity.to_string(), e))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::{
        md_parser::tests::assert_parse_eq,
        unit::{Nominal, Unit, Volume},
    };

    macro_rules! assert_quantity {
        ( $txt:expr, $unit:expr, $amount:expr ) => {
            let s: &str = $txt;
            assert_parse_eq!(
                Quantity::from_str(s),
                Quantity {
                    unit: $unit.clone(),
                    amount: $amount,
                }
            );
        };
    }

    macro_rules! assert_quantity_of {
        ( $unitty:ty, $txt:expr, $unit:expr, $amount:expr ) => {
            let s: &str = $txt;
            let unit_of_ty: $unitty = $unit;
            assert_parse_eq!(
                QuantityOf::<$unitty>::from_str(s),
                QuantityOf::<$unitty> {
                    unit: unit_of_ty.clone(),
                    amount: $amount,
                }
            );
        };
    }

    #[test]
    fn parse_quantity() {
        assert_quantity!("1", Unit::Nominal(Nominal), 1.);
        assert_quantity!("10 g", Unit::Mass(Mass::Gram), 10.);
        assert_quantity!("50 mL", Unit::Volume(Volume::Milliliter), 50.);
        assert_quantity!("50ML", Unit::Volume(Volume::Milliliter), 50.);
        assert_quantity!("  50.111 Ml    ", Unit::Volume(Volume::Milliliter), 50.111);
        assert_quantity!("2.5cm", Unit::Distance(Distance::Centimeter), 2.5);
        assert_quantity!("180°C", Unit::Temperature(Temperature::Celsius), 180.);
        assert_quantity!("60 sec.", Unit::Time(Time::Second), 60.);
        assert_quantity!("  0.5 bunch    ", Unit::Custom("bunch".to_string()), 0.5);
    }

    #[test]
    fn parse_quantity_failures() {
        // The empty string does not represent a valid quantity.
        assert!(Quantity::from_str("").is_err());
        // The decimal separator should be a '.', not a ','".
        assert!(Quantity::from_str("1,5 g").is_err());
        // Invalid float.
        assert!(Quantity::from_str("1.5.1 g").is_err());
    }

    #[test]
    fn parse_quantity_of() {
        assert_quantity_of!(Nominal, "1", Nominal, 1.);
        assert_quantity_of!(Volume, "50 mL", Volume::Milliliter, 50.);
        assert_quantity_of!(Volume, "50ML", Volume::Milliliter, 50.);
        assert_quantity_of!(Volume, "  50 Ml    ", Volume::Milliliter, 50.);
        assert_quantity_of!(Temperature, "180°C", Temperature::Celsius, 180.);
    }

    #[test]
    fn parse_quantity_of_failures() {
        // The empty string does not represent a valid quantity of anything.
        assert!(QuantityOf::<Mass>::from_str("").is_err());
        // The decimal separator should be a '.', not a ','".
        assert_eq!(
            QuantityOf::<Mass>::from_str("1,0 g").unwrap_err(),
            ParseQuantityOfError::InvalidAmount(
                "1,0".to_string(),
                "1,0".parse::<f32>().unwrap_err()
            )
        );
        // 'mL' does not represent a mass.
        assert_eq!(
            QuantityOf::<Mass>::from_str("1 mL").unwrap_err(),
            ParseQuantityOfError::InvalidUnit("mL".to_string())
        );
        // Spaces around and between amount and unit should not change error string.
        assert_eq!(
            QuantityOf::<Mass>::from_str("    1mL  ").unwrap_err(),
            ParseQuantityOfError::InvalidUnit("mL".to_string())
        );
    }
}
