mod conv;
mod fit;
mod offset;
mod piece;
mod read;
mod write;

use ffb_pattern::Panel;

pub use read::import;
pub use write::export;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavour {
    Aama,
    Plain,
}

impl Flavour {
    pub fn name(self) -> &'static str {
        match self {
            Flavour::Aama => "aama",
            Flavour::Plain => "plain",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub flavour: Option<Flavour>,
    pub scale: Option<f64>,
    pub fit_tolerance: f64,
    pub sew_allowance: bool,
    pub smooth_facets: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        ImportOptions { flavour: None, scale: None, fit_tolerance: 0.05, sew_allowance: true, smooth_facets: true }
    }
}

#[derive(Debug, Clone)]
pub struct Imported {
    pub panels: Vec<Panel>,
    pub flavour: Flavour,
    pub scale: f64,
    pub keep_positions: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExportPanel<'a> {
    pub panel: &'a Panel,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub flavour: Flavour,
    pub seam_allowance: bool,
    pub notches: bool,
    pub grain_line: bool,
    pub inner_lines: bool,
    pub labels: bool,
    pub tolerance: f64,
    pub unit_mm: f64,
    pub style: String,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions { flavour: Flavour::Aama, seam_allowance: true, notches: true, grain_line: true, inner_lines: true, labels: true, tolerance: 0.1, unit_mm: 1.0, style: "formfabric".into() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Exported {
    pub text: String,
    pub panels: usize,
    pub notches: usize,
    pub inner_lines: usize,
    pub grain_lines: usize,
    pub vertices: usize,
}
