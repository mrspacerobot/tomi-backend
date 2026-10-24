//! A graph recipe as the artifact writes it: its lines, steps and yield, read once into types.

use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize)]
pub struct SourceRecipe {
    pub ingredients: Vec<SourceLine>,
    pub steps: Vec<SourceStep>,
    pub yields: RecipeAmount,
}

#[derive(Debug, Deserialize)]
pub struct SourceLine {
    pub part: String,
    pub thing: String,
    pub amount: RecipeAmount,
    pub forms: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SourceStep {
    pub text: String,
    pub time: Option<StepTime>,
    pub temperature: Option<StepTemperature>,
    pub safety_points: Vec<String>,
}

/// The units the graph writes an amount in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Gram,
    Kilogram,
    Millilitre,
    Litre,
    Teaspoon,
    Tablespoon,
    Piece,
    Clove,
    Pinch,
    Sprig,
    Slice,
    Leaf,
    Serving,
}

/// What a ratio measures by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    Weight,
    Volume,
    Count,
}

impl Unit {
    pub fn measure(self) -> Option<Measure> {
        match self {
            Unit::Gram | Unit::Kilogram => Some(Measure::Weight),
            Unit::Millilitre | Unit::Litre | Unit::Teaspoon | Unit::Tablespoon => Some(Measure::Volume),
            Unit::Serving => None,
            _ => Some(Measure::Count),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeAmount {
    #[serde(serialize_with = "plain_number")]
    pub value: f64,
    pub unit: Unit,
}

/// Minutes or seconds, as a span or a single number.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StepTime {
    Span {
        #[serde(serialize_with = "plain_number")]
        from: f64,
        #[serde(serialize_with = "plain_number")]
        to: f64,
        unit: String,
    },
    Exactly {
        #[serde(serialize_with = "plain_number")]
        value: f64,
        unit: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepTemperature {
    #[serde(serialize_with = "plain_number")]
    pub value: f64,
    pub unit: String,
}

/// A whole number is written without a fraction, so 5 stays 5.
fn plain_number<S: Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return serializer.serialize_i64(*value as i64);
    }
    serializer.serialize_f64(*value)
}
