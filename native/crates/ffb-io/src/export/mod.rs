pub mod cache;
pub mod fbx;
pub mod gltf;
pub mod obj;
pub mod png;
pub mod thick;
pub mod usd;
pub mod uv;

use serde_json::{json, Value};

pub const MAX_BYTES: usize = 1 << 30;
pub const MAX_FRAMES: usize = 2000;

pub type V3 = [f32; 3];

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    pub color: [f32; 4],
    pub roughness: f32,
    pub metalness: f32,
    pub opacity: f32,
    pub base_color_map: Option<String>,
    pub normal_map: Option<String>,
    pub double_sided: bool,
}

impl Default for Material {
    fn default() -> Self {
        Material { name: "material".into(), color: [0.78, 0.78, 0.78, 1.0], roughness: 0.85, metalness: 0.0, opacity: 1.0, base_color_map: None, normal_map: None, double_sided: true }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub material: usize,
    pub first: usize,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Object {
    pub name: String,
    pub positions: Vec<V3>,
    pub normals: Vec<V3>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scene {
    pub objects: Vec<Object>,
    pub materials: Vec<Material>,
}

impl Scene {
    pub fn vertex_count(&self) -> usize {
        self.objects.iter().map(|o| o.positions.len()).sum()
    }

    pub fn triangle_count(&self) -> usize {
        self.objects.iter().map(|o| o.indices.len() / 3).sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SrcObject {
    pub name: String,
    pub positions: Vec<V3>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub front: usize,
    pub back: usize,
    pub side: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Thickness {
    pub amount: f32,
    pub sides: bool,
}

pub struct Built {
    pub scene: Scene,
    extrusions: Vec<Option<thick::Extrusion>>,
    sources: Vec<(Vec<u32>, usize)>,
    amount: f32,
}

impl Built {
    pub fn raw_vertex_count(&self) -> usize {
        self.sources.iter().map(|s| s.1).sum()
    }

    pub fn frame(&self, raw: &[V3]) -> Result<Vec<V3>, String> {
        if raw.len() != self.raw_vertex_count() {
            return Err(format!("cache frame has {} points but the garment has {}", raw.len(), self.raw_vertex_count()));
        }
        let mut out = Vec::with_capacity(self.scene.vertex_count());
        let mut at = 0;
        for (i, (_, n)) in self.sources.iter().enumerate() {
            let part = &raw[at..at + n];
            at += n;
            match &self.extrusions[i] {
                Some(e) => out.extend(e.positions(part, self.amount)),
                None => out.extend_from_slice(part),
            }
        }
        Ok(out)
    }
}

pub fn build(objects: &[SrcObject], materials: Vec<Material>, thickness: Option<&Thickness>) -> Result<Built, String> {
    let mut scene = Scene { objects: Vec::new(), materials };
    let mut extrusions = Vec::new();
    let mut sources = Vec::new();
    for o in objects {
        if o.uvs.len() != o.positions.len() {
            return Err(format!("{} has {} UVs for {} vertices", o.name, o.uvs.len(), o.positions.len()));
        }
        if o.indices.iter().any(|i| *i as usize >= o.positions.len()) || o.indices.len() % 3 != 0 {
            return Err(format!("{} has invalid triangle indices", o.name));
        }
        sources.push((o.indices.clone(), o.positions.len()));
        match thickness {
            Some(t) if t.amount > 0.0 => {
                let (object, e) = thick::extrude(o, t);
                extrusions.push(Some(e));
                scene.objects.push(object);
            }
            _ => {
                extrusions.push(None);
                let mut object = Object {
                    name: o.name.clone(),
                    positions: o.positions.clone(),
                    normals: Vec::new(),
                    uvs: o.uvs.clone(),
                    indices: o.indices.clone(),
                    groups: vec![Group { material: o.front, first: 0, count: o.indices.len() / 3 }],
                };
                object.normals = vertex_normals(&object.positions, &object.indices);
                scene.objects.push(object);
            }
        }
    }
    Ok(Built { scene, extrusions, sources, amount: thickness.map_or(0.0, |t| t.amount) })
}

pub fn vertex_normals(positions: &[V3], indices: &[u32]) -> Vec<V3> {
    let mut acc = vec![[0.0f32; 3]; positions.len()];
    for t in indices.chunks_exact(3) {
        let (a, b, c) = (positions[t[0] as usize], positions[t[1] as usize], positions[t[2] as usize]);
        let n = cross(sub(b, a), sub(c, a));
        for i in t {
            let s = &mut acc[*i as usize];
            s[0] += n[0];
            s[1] += n[1];
            s[2] += n[2];
        }
    }
    acc.into_iter().map(normalize).collect()
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn normalize(v: V3) -> V3 {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-20 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Obj,
    Gltf,
    Glb,
    Fbx,
    Usd,
    Mdd,
    Pc2,
}

impl Format {
    pub const ALL: [Format; 7] = [Format::Obj, Format::Gltf, Format::Glb, Format::Fbx, Format::Usd, Format::Mdd, Format::Pc2];

    pub fn parse(name: &str) -> Option<Format> {
        Some(match name {
            "obj" => Format::Obj,
            "gltf" => Format::Gltf,
            "glb" => Format::Glb,
            "fbx" => Format::Fbx,
            "usd" | "usda" => Format::Usd,
            "mdd" => Format::Mdd,
            "pc2" => Format::Pc2,
            _ => return None,
        })
    }

    pub fn id(self) -> &'static str {
        match self {
            Format::Obj => "obj",
            Format::Gltf => "gltf",
            Format::Glb => "glb",
            Format::Fbx => "fbx",
            Format::Usd => "usd",
            Format::Mdd => "mdd",
            Format::Pc2 => "pc2",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Format::Usd => "usda",
            other => other.id(),
        }
    }

    pub fn accepts(self, extension: &str) -> bool {
        let e = extension.to_ascii_lowercase();
        match self {
            Format::Usd => e == "usda" || e == "usd",
            other => e == other.id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub stem: String,
    pub scale: f64,
    pub z_up: bool,
    pub merged: bool,
    pub materials: bool,
    pub uv: bool,
    pub fps: f32,
    pub start_frame: u32,
    pub binary_fbx: bool,
    pub meters_per_unit: f64,
}

impl Default for Options {
    fn default() -> Self {
        Options { stem: "export".into(), scale: 1.0, z_up: false, merged: false, materials: true, uv: true, fps: 30.0, start_frame: 0, binary_fbx: false, meters_per_unit: 0.001 }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Animation {
    pub frames: Vec<Vec<V3>>,
}

pub struct File {
    pub name: String,
    pub bytes: Vec<u8>,
}

fn convert(p: V3, scale: f64, z_up: bool) -> V3 {
    let s = scale as f32;
    let q = [p[0] * s, p[1] * s, p[2] * s];
    if z_up {
        [q[0], -q[2], q[1]]
    } else {
        q
    }
}

fn convert_scene(scene: &Scene, o: &Options) -> Scene {
    let mut out = scene.clone();
    for obj in out.objects.iter_mut() {
        for p in obj.positions.iter_mut() {
            *p = convert(*p, o.scale, o.z_up);
        }
        obj.normals = obj.normals.iter().map(|n| convert(*n, 1.0, o.z_up)).collect();
    }
    out
}

fn merge(scene: &Scene) -> Scene {
    let mut merged = Object { name: "garment".into(), ..Object::default() };
    for obj in &scene.objects {
        let base = merged.positions.len() as u32;
        let tri_base = merged.indices.len() / 3;
        merged.positions.extend_from_slice(&obj.positions);
        merged.normals.extend_from_slice(&obj.normals);
        merged.uvs.extend_from_slice(&obj.uvs);
        merged.indices.extend(obj.indices.iter().map(|i| i + base));
        for g in &obj.groups {
            match merged.groups.last_mut() {
                Some(last) if last.material == g.material && last.first + last.count == tri_base + g.first => last.count += g.count,
                _ => merged.groups.push(Group { material: g.material, first: tri_base + g.first, count: g.count }),
            }
        }
    }
    Scene { objects: vec![merged], materials: scene.materials.clone() }
}

pub fn estimate_bytes(scene: &Scene, frames: usize, format: Format) -> usize {
    let v = scene.vertex_count();
    let t = scene.triangle_count();
    let per_frame = v * 12;
    let base = v * 40 + t * 24;
    match format {
        Format::Mdd | Format::Pc2 => frames * per_frame + 64,
        Format::Obj => base * 2 + frames.saturating_sub(1).max(if frames > 0 { 1 } else { 0 }) * (v * 30 + t * 30),
        Format::Usd => base + frames * v * 36,
        Format::Fbx => base * 2 + frames * v * 40,
        Format::Gltf | Format::Glb => base + frames * per_frame,
    }
}

pub fn write(format: Format, scene: &Scene, animation: Option<&Animation>, options: &Options) -> Result<Vec<File>, String> {
    if scene.objects.is_empty() || scene.vertex_count() == 0 {
        return Err("there is nothing to export".into());
    }
    let frames = animation.map_or(0, |a| a.frames.len());
    if frames > MAX_FRAMES {
        return Err(format!("{frames} frames is over the limit of {MAX_FRAMES}"));
    }
    if matches!(format, Format::Mdd | Format::Pc2) && frames == 0 {
        return Err("a point cache needs animation frames; record a cache first".into());
    }
    let estimate = estimate_bytes(scene, frames, format);
    if estimate > MAX_BYTES {
        return Err(format!("the file would be about {} MiB, over the {} MiB limit; use fewer frames or a coarser mesh", estimate >> 20, MAX_BYTES >> 20));
    }
    let mut base = scene.clone();
    if options.merged {
        base = merge(&base);
    }
    let base = convert_scene(&base, options);
    let anim: Option<Animation> = animation.filter(|a| !a.frames.is_empty()).map(|a| Animation { frames: a.frames.iter().map(|f| f.iter().map(|p| convert(*p, options.scale, options.z_up)).collect()).collect() });
    if let Some(a) = &anim {
        if a.frames.iter().any(|f| f.len() != base.vertex_count()) {
            return Err("an animation frame does not match the exported vertex count".into());
        }
    }
    let files = match format {
        Format::Obj => obj::write(&base, anim.as_ref(), options),
        Format::Gltf => gltf::write(&base, anim.as_ref(), options, false),
        Format::Glb => gltf::write(&base, anim.as_ref(), options, true),
        Format::Fbx => fbx::write(&base, anim.as_ref(), options),
        Format::Usd => usd::write(&base, anim.as_ref(), options),
        Format::Mdd => Ok(cache::mdd(anim.as_ref().ok_or("no frames")?, options)),
        Format::Pc2 => Ok(cache::pc2(anim.as_ref().ok_or("no frames")?, options)),
    }?;
    let total: usize = files.iter().map(|f| f.bytes.len()).sum();
    if total > MAX_BYTES {
        return Err(format!("the output is {} MiB, over the {} MiB limit", total >> 20, MAX_BYTES >> 20));
    }
    Ok(files)
}

pub fn format_options() -> Value {
    json!({
        "scale": {"type": "number", "minimum": 0.000001, "maximum": 1000000.0, "default": 1.0, "description": "Multiplier applied on top of units."},
        "units": {"type": "string", "enum": ["mm", "cm", "m", "in"], "default": "mm", "description": "Unit of the numbers in the file. The engine works in millimetres."},
        "axis": {"type": "string", "enum": ["y_up", "z_up"], "default": "y_up", "description": "y_up keeps the engine axes. z_up rotates to Z up, Y forward (Blender, 3ds Max)."},
        "single_object": {"type": "boolean", "default": false, "description": "Merge every panel into one object."},
        "thickness": {"type": "boolean", "description": "Extrude the back face and the border strips using thickness.set. Defaults to the project setting."},
        "uv": {"type": "string", "enum": ["packed", "tiled", "none"], "default": "packed", "description": "packed uses the island layout (0..1 and UDIM tiles along U). tiled uses grain aligned millimetres divided by the fabric map scale, so texture maps repeat."},
        "materials": {"type": "boolean", "default": true, "description": "Write fabric materials: base colour, roughness, metalness, opacity and texture paths."},
        "animation": {"type": "boolean", "default": false, "description": "Include the active cache layer as animation."},
        "cache": {"type": "integer", "minimum": 1, "description": "Cache layer id. Defaults to the active cache."},
        "start": {"type": "integer", "minimum": 0, "description": "First cache frame (inclusive)."},
        "end": {"type": "integer", "minimum": 0, "description": "Last cache frame (inclusive)."},
        "step": {"type": "integer", "minimum": 1, "default": 1, "description": "Frame step."}
    })
}

pub fn formats() -> Value {
    let common = format_options();
    let with = |extra: Value| -> Value {
        let mut m = common.as_object().cloned().unwrap_or_default();
        if let Some(e) = extra.as_object() {
            for (k, v) in e {
                m.insert(k.clone(), v.clone());
            }
        }
        Value::Object(m)
    };
    json!([
        {"id": "obj", "extension": "obj", "files": ["obj", "mtl"], "animation": "frame sequence: stem_0001.obj, stem_0002.obj and so on, one shared mtl", "options": with(json!({}))},
        {"id": "gltf", "extension": "gltf", "files": ["gltf", "bin"], "animation": "morph targets with a weights animation (one target per frame)", "options": with(json!({}))},
        {"id": "glb", "extension": "glb", "files": ["glb"], "animation": "morph targets with a weights animation (one target per frame)", "options": with(json!({}))},
        {"id": "fbx", "extension": "fbx", "files": ["fbx"], "animation": "blend shapes (one shape per frame) with a DeformPercent curve; own FBX 7.4 writer", "options": with(json!({"binary": {"type": "boolean", "default": false, "description": "Write binary FBX 7.4 instead of ASCII. Blender's importer reads binary only."}}))},
        {"id": "usd", "extension": "usda", "files": ["usda"], "animation": "time sampled points on each Mesh", "options": with(json!({}))},
        {"id": "mdd", "extension": "mdd", "files": ["mdd"], "animation": "point cache (needs a cache)", "options": with(json!({}))},
        {"id": "pc2", "extension": "pc2", "files": ["pc2"], "animation": "point cache (needs a cache)", "options": with(json!({}))},
        {"id": "abc", "extension": "abc", "files": [], "available": false, "note": "Alembic needs an Ogawa writer; deferred. Export USD or glTF animation, or MDD/PC2, and convert."}
    ])
}
