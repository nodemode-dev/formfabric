use ffb_io::export::uv::{self, Island};
use ffb_io::export::{Material, SrcObject};
use ffb_mesh::layout;
use ffb_pattern::output::UvIsland;
use ffb_pattern::Garment;
use serde_json::Value;

use crate::{CommandError, Session};

pub(super) fn bad(message: impl Into<String>) -> CommandError {
    CommandError::new("bad_args", message)
}

pub(super) fn flag(a: &Value, key: &str) -> Option<bool> {
    a.get(key).filter(|v| !v.is_null()).and_then(Value::as_bool)
}

pub(super) fn number(a: &Value, key: &str, lo: f64, hi: f64) -> Result<Option<f64>, CommandError> {
    match a.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => v.as_f64().filter(|x| x.is_finite() && *x >= lo && *x <= hi).map(Some).ok_or_else(|| bad(format!("{key} must be a number between {lo} and {hi}"))),
    }
}

pub(super) struct PanelGeo {
    pub id: u32,
    pub name: String,
    pub indices: Vec<u32>,
    pub aligned: Vec<[f64; 2]>,
    pub pos3: Vec<[f32; 3]>,
    pub fabric: Option<u32>,
    pub map_scale: [f64; 2],
}

pub(super) fn grain_angle(panel: &ffb_pattern::Panel) -> f64 {
    ffb_pattern::grain::effective(panel).map_or(90.0, |(line, _)| ffb_pattern::grain::angle_of(line))
}

pub(super) fn collect(s: &mut Session, filter: Option<&[u32]>) -> Result<Vec<PanelGeo>, CommandError> {
    let ids: Vec<u32> = match filter {
        Some(list) => list.to_vec(),
        None => s.garment.panels.iter().filter(|p| !p.state.disabled).map(|p| p.id).collect(),
    };
    if ids.is_empty() {
        return Err(CommandError::new("bad_args", "the pattern has no panels to export"));
    }
    let mut out = Vec::new();
    for id in ids {
        let index = s.garment.index_of(id).ok_or_else(|| CommandError::new("not_found", format!("no panel with id {id}")))?;
        let built = s.mesh.ensure(&s.garment.panels[index], false).built.map_err(|e| CommandError::new("invalid_geometry", e))?;
        let pos3 = crate::sim_cmds::current_positions(s, id)?;
        let panel = &s.garment.panels[index];
        let flat: Vec<[f64; 2]> = built.mesh.positions.clone();
        let fabric = panel.fabric.or(s.garment.default_fabric);
        let map_scale = fabric.and_then(|f| s.garment.fabrics.iter().find(|x| x.id == f)).map_or([100.0, 100.0], |f| f.look.front.map_scale);
        out.push(PanelGeo {
            id,
            name: if panel.name.is_empty() { format!("panel_{id}") } else { panel.name.clone() },
            indices: layout::indices(&built.mesh, layout::mirrored_flat(panel)),
            aligned: uv::aligned(&flat, grain_angle(panel)),
            pos3,
            fabric,
            map_scale,
        });
    }
    Ok(out)
}

pub(super) fn stored_islands(g: &Garment, geos: &[PanelGeo]) -> Option<Vec<Island>> {
    let mut out = Vec::new();
    for geo in geos {
        let i = g.output.islands.iter().find(|i| i.panel == geo.id).filter(|i| i.scale > 0.0 && i.scale.is_finite())?;
        out.push(Island { panel: i.panel, offset: i.offset, rotation: i.rotation, scale: i.scale, tile: i.tile });
    }
    Some(out)
}

pub(super) fn auto_layout(geos: &[PanelGeo], padding: f32, tiles: u32) -> Vec<Island> {
    let sizes: Vec<(u32, [f64; 2])> = geos.iter().map(|g| (g.id, uv::island_size(&g.aligned, 0.0))).collect();
    uv::pack(&sizes, &[], padding, tiles)
}

pub(super) fn effective_islands(g: &Garment, geos: &[PanelGeo]) -> (Vec<Island>, bool) {
    match stored_islands(g, geos) {
        Some(list) => (list, true),
        None => (auto_layout(geos, g.output.uv_padding, 1), false),
    }
}

pub(super) fn to_stored(i: &Island) -> UvIsland {
    UvIsland { panel: i.panel, offset: i.offset, rotation: i.rotation, scale: i.scale, tile: i.tile }
}

pub(super) fn tile_count(islands: &[Island]) -> u32 {
    islands.iter().map(|i| i.tile + 1).max().unwrap_or(1)
}

pub(super) fn uvs_for(geo: &PanelGeo, island: &Island, mode: &str) -> Vec<[f32; 2]> {
    match mode {
        "tiled" => uv::tiled_uvs(&geo.aligned, geo.map_scale),
        "none" => vec![[0.0, 0.0]; geo.aligned.len()],
        _ => uv::island_uvs(&geo.aligned, island),
    }
}

pub(super) struct MaterialSet {
    pub materials: Vec<Material>,
    pub per_panel: Vec<[usize; 3]>,
}

pub(super) fn materials(g: &Garment, geos: &[PanelGeo], thick: bool) -> MaterialSet {
    let mut set = MaterialSet { materials: Vec::new(), per_panel: Vec::new() };
    let mut keys: Vec<(Option<u32>, u8)> = Vec::new();
    let active = g.active_colorway.and_then(|c| g.colorways.iter().find(|w| w.id == c));
    for geo in geos {
        let mut slots = [0usize; 3];
        for (role, slot) in slots.iter_mut().enumerate() {
            let key = (geo.fabric, role as u8);
            let existing = keys.iter().position(|k| *k == key);
            *slot = match existing {
                Some(i) => i,
                None => {
                    let fabric = geo.fabric.and_then(|f| g.fabrics.iter().find(|x| x.id == f));
                    let mut m = Material { double_sided: !thick, ..Material::default() };
                    if let Some(f) = fabric {
                        let surface = match role {
                            1 => f.look.back.as_ref().unwrap_or(&f.look.front),
                            2 => f.look.side.as_ref().unwrap_or(&f.look.front),
                            _ => &f.look.front,
                        };
                        let separate = (role == 1 && f.look.back.is_some()) || (role == 2 && f.look.side.is_some());
                        m.name = match role {
                            1 if separate => format!("{}_back", f.name),
                            2 if separate => format!("{}_side", f.name),
                            _ => f.name.clone(),
                        };
                        m.color = surface.color;
                        if role == 0 || !separate {
                            if let Some(c) = active.and_then(|w| w.fabric_colors.get(&f.id)) {
                                m.color = *c;
                            }
                        }
                        m.roughness = surface.roughness;
                        m.metalness = surface.metalness;
                        m.opacity = surface.opacity;
                        m.base_color_map = surface.maps.base_color.clone();
                        m.normal_map = surface.maps.normal.clone();
                    } else {
                        m.name = "fabric".into();
                    }
                    set.materials.push(m);
                    keys.push(key);
                    set.materials.len() - 1
                }
            };
        }
        set.per_panel.push(slots);
    }
    dedupe(&mut set);
    set
}

pub(super) fn dedupe(set: &mut MaterialSet) {
    let mut unique: Vec<Material> = Vec::new();
    let mut remap = Vec::new();
    for m in &set.materials {
        match unique.iter().position(|u| u == m) {
            Some(i) => remap.push(i),
            None => {
                unique.push(m.clone());
                remap.push(unique.len() - 1);
            }
        }
    }
    for slots in set.per_panel.iter_mut() {
        for s in slots.iter_mut() {
            *s = remap[*s];
        }
    }
    set.materials = unique;
}

pub(super) fn source_objects(geos: &[PanelGeo], islands: &[Island], mats: &MaterialSet, mode: &str) -> Vec<SrcObject> {
    let mut used_names: Vec<String> = Vec::new();
    geos.iter()
        .enumerate()
        .map(|(i, geo)| {
            let mut name = geo.name.clone();
            if used_names.contains(&name) {
                name = format!("{}_{}", name, geo.id);
            }
            used_names.push(name.clone());
            SrcObject { name, positions: geo.pos3.clone(), uvs: uvs_for(geo, &islands[i], mode), indices: geo.indices.clone(), front: mats.per_panel[i][0], back: mats.per_panel[i][1], side: mats.per_panel[i][2] }
        })
        .collect()
}
