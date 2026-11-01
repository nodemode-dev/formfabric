use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    #[default]
    Mm,
    Cm,
    In,
}

impl Units {
    pub const ALL: [Units; 3] = [Units::Mm, Units::Cm, Units::In];

    pub fn parse(text: &str) -> Option<Units> {
        match text.trim().to_ascii_lowercase().as_str() {
            "mm" => Some(Units::Mm),
            "cm" => Some(Units::Cm),
            "in" | "inch" | "inches" => Some(Units::In),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Units::Mm => "mm",
            Units::Cm => "cm",
            Units::In => "in",
        }
    }

    pub fn mm_per_unit(self) -> f64 {
        match self {
            Units::Mm => 1.0,
            Units::Cm => 10.0,
            Units::In => 25.4,
        }
    }

    pub fn from_mm(self, mm: f64) -> f64 {
        mm / self.mm_per_unit()
    }

    pub fn to_mm(self, value: f64) -> f64 {
        value * self.mm_per_unit()
    }
}
