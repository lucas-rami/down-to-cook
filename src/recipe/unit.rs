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
