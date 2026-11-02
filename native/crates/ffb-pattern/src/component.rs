use serde::{Deserialize, Serialize};

use crate::sew::{end_valid, side_length};
use crate::{Garment, SeamEnd};

pub const DEFAULT_TOLERANCE_MM: f64 = 5.0;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Port {
    pub name: String,
    pub spans: Vec<SeamEnd>,
    pub groups: Vec<usize>,
    pub length_mm: f64,
    pub tolerance_mm: f64,
}

impl Port {
    pub fn runs(&self) -> Vec<&[SeamEnd]> {
        if self.groups.is_empty() || self.groups.iter().sum::<usize>() != self.spans.len() || self.groups.contains(&0) {
            return vec![self.spans.as_slice()];
        }
        let mut at = 0;
        self.groups
            .iter()
            .map(|n| {
                at += n;
                &self.spans[at - n..at]
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Component {
    pub id: u32,
    pub name: String,
    pub panels: Vec<u32>,
    pub ports: Vec<Port>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PortRef {
    pub component: u32,
    pub port: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Join {
    pub id: u32,
    pub a: PortRef,
    pub b: PortRef,
    pub reversed: bool,
    pub seams: Vec<u32>,
}

pub fn next_component_id(garment: &Garment) -> u32 {
    garment.components.iter().map(|c| c.id).max().unwrap_or(0).saturating_add(1)
}

pub fn next_join_id(garment: &Garment) -> u32 {
    garment.joins.iter().map(|j| j.id).max().unwrap_or(0).saturating_add(1)
}

impl Garment {
    pub fn component(&self, id: u32) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }

    pub fn component_of_panel(&self, panel: u32) -> Option<&Component> {
        self.components.iter().find(|c| c.panels.contains(&panel))
    }

    pub fn port(&self, r: &PortRef) -> Option<&Port> {
        self.component(r.component).and_then(|c| c.ports.iter().find(|p| p.name == r.port))
    }

    pub fn join_of_port(&self, r: &PortRef) -> Option<&Join> {
        self.joins.iter().find(|j| j.a == *r || j.b == *r)
    }
}

pub fn port_length(garment: &Garment, port: &Port) -> Option<f64> {
    side_length(garment, &port.spans)
}

pub fn port_valid(garment: &Garment, port: &Port) -> bool {
    !port.spans.is_empty() && port.spans.iter().all(|e| end_valid(garment, e))
}

pub fn internal_seams(garment: &Garment, component: &Component) -> Vec<u32> {
    garment
        .seams
        .iter()
        .filter(|s| s.a.iter().chain(s.b.iter()).all(|e| component.panels.contains(&e.panel)))
        .map(|s| s.id)
        .collect()
}

pub fn prune(garment: &mut Garment) {
    let live: Vec<u32> = garment.panels.iter().map(|p| p.id).collect();
    let mut components = std::mem::take(&mut garment.components);
    for c in components.iter_mut() {
        c.panels.retain(|p| live.contains(p));
        for port in c.ports.iter_mut() {
            port.spans.retain(|e| c.panels.contains(&e.panel));
        }
        c.ports.retain(|p| !p.spans.is_empty());
    }
    components.retain(|c| !c.panels.is_empty());
    garment.components = components;
    let seams: Vec<u32> = garment.seams.iter().map(|s| s.id).collect();
    let mut joins = std::mem::take(&mut garment.joins);
    for j in joins.iter_mut() {
        j.seams.retain(|s| seams.contains(s));
    }
    joins.retain(|j| !j.seams.is_empty() && garment.port(&j.a).is_some() && garment.port(&j.b).is_some());
    garment.joins = joins;
}
