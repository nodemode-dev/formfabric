use serde_json::{json, Value};

use super::{Animation, File, Options, Scene, V3};

struct Buf {
    bytes: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
}

impl Buf {
    fn view(&mut self, data: &[u8], target: Option<u32>) -> usize {
        while !self.bytes.len().is_multiple_of(4) {
            self.bytes.push(0);
        }
        let mut v = json!({"buffer": 0, "byteOffset": self.bytes.len(), "byteLength": data.len()});
        if let Some(t) = target {
            v["target"] = json!(t);
        }
        self.bytes.extend_from_slice(data);
        self.views.push(v);
        self.views.len() - 1
    }

    fn accessor(&mut self, data: &[u8], target: Option<u32>, component: u32, count: usize, kind: &str, bounds: Option<(Vec<f32>, Vec<f32>)>) -> usize {
        let view = self.view(data, target);
        let mut a = json!({"bufferView": view, "componentType": component, "count": count, "type": kind});
        if let Some((lo, hi)) = bounds {
            a["min"] = json!(lo);
            a["max"] = json!(hi);
        }
        self.accessors.push(a);
        self.accessors.len() - 1
    }

    fn vec3(&mut self, v: &[V3], target: Option<u32>, bounds: bool) -> usize {
        let mut data = Vec::with_capacity(v.len() * 12);
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in v {
            for k in 0..3 {
                data.extend_from_slice(&p[k].to_le_bytes());
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        let b = (bounds && !v.is_empty()).then(|| (lo.to_vec(), hi.to_vec()));
        self.accessor(&data, target, 5126, v.len(), "VEC3", b)
    }

    fn floats(&mut self, v: &[f32], bounds: Option<(f32, f32)>) -> usize {
        let data: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
        self.accessor(&data, None, 5126, v.len(), "SCALAR", bounds.map(|(a, b)| (vec![a], vec![b])))
    }
}

fn sanitize_uri(path: &str) -> String {
    path.replace('\\', "/").replace(' ', "%20")
}

pub fn write(scene: &Scene, animation: Option<&Animation>, o: &Options, glb: bool) -> Result<Vec<File>, String> {
    let mut buf = Buf { bytes: Vec::new(), views: Vec::new(), accessors: Vec::new() };
    let mut images: Vec<Value> = Vec::new();
    let mut textures: Vec<Value> = Vec::new();
    let mut materials: Vec<Value> = Vec::new();
    let texture_of =|path: &str, images: &mut Vec<Value>, textures: &mut Vec<Value>| -> usize {
        let uri = sanitize_uri(path);
        if let Some(i) = images.iter().position(|m| m["uri"] == json!(uri)) {
            return i;
        }
        images.push(json!({"uri": uri}));
        textures.push(json!({"source": images.len() - 1, "sampler": 0}));
        textures.len() - 1
    };
    if o.materials {
        for m in &scene.materials {
            let mut pbr = json!({"baseColorFactor": [m.color[0], m.color[1], m.color[2], m.color[3] * m.opacity], "metallicFactor": m.metalness, "roughnessFactor": m.roughness});
            if let (true, Some(t)) = (o.uv, &m.base_color_map) {
                pbr["baseColorTexture"] = json!({"index": texture_of(t, &mut images, &mut textures)});
            }
            let mut mat = json!({"name": m.name, "pbrMetallicRoughness": pbr, "doubleSided": m.double_sided});
            if let (true, Some(t)) = (o.uv, &m.normal_map) {
                mat["normalTexture"] = json!({"index": texture_of(t, &mut images, &mut textures)});
            }
            if m.opacity * m.color[3] < 0.999 {
                mat["alphaMode"] = json!("BLEND");
            }
            materials.push(mat);
        }
    }
    let frames = animation.map_or(0, |a| a.frames.len());
    let mut meshes = Vec::new();
    let mut nodes = Vec::new();
    let mut at = 0usize;
    let mut weight_nodes = Vec::new();
    for (oi, obj) in scene.objects.iter().enumerate() {
        let n = obj.positions.len();
        let pos = buf.vec3(&obj.positions, Some(34962), true);
        let nor = buf.vec3(&obj.normals, Some(34962), false);
        let uv = if o.uv {
            let mut d = Vec::with_capacity(n * 8);
            for t in &obj.uvs {
                d.extend_from_slice(&t[0].to_le_bytes());
                d.extend_from_slice(&(1.0 - t[1]).to_le_bytes());
            }
            Some(buf.accessor(&d, Some(34962), 5126, n, "VEC2", None))
        } else {
            None
        };
        let mut targets = Vec::new();
        if let Some(a) = animation {
            for f in &a.frames {
                let delta: Vec<V3> = f[at..at + n].iter().zip(&obj.positions).map(|(p, b)| [p[0] - b[0], p[1] - b[1], p[2] - b[2]]).collect();
                targets.push(json!({"POSITION": buf.vec3(&delta, None, true)}));
            }
        }
        let mut prims = Vec::new();
        for g in &obj.groups {
            let slice = &obj.indices[g.first * 3..(g.first + g.count) * 3];
            let data: Vec<u8> = slice.iter().flat_map(|i| i.to_le_bytes()).collect();
            let idx = buf.accessor(&data, Some(34963), 5125, slice.len(), "SCALAR", None);
            let mut attrs = json!({"POSITION": pos, "NORMAL": nor});
            if let Some(u) = uv {
                attrs["TEXCOORD_0"] = json!(u);
            }
            let mut p = json!({"attributes": attrs, "indices": idx, "mode": 4});
            if o.materials && g.material < materials.len() {
                p["material"] = json!(g.material);
            }
            if !targets.is_empty() {
                p["targets"] = json!(targets);
            }
            prims.push(p);
        }
        let mut mesh = json!({"name": obj.name, "primitives": prims});
        if frames > 0 {
            mesh["weights"] = json!(vec![0.0f32; frames]);
            weight_nodes.push(oi);
        }
        meshes.push(mesh);
        nodes.push(json!({"name": obj.name, "mesh": oi}));
        at += n;
    }
    let mut root = json!({
        "asset": {"version": "2.0", "generator": "formfabric"},
        "scene": 0,
        "scenes": [{"name": "garment", "nodes": (0..nodes.len()).collect::<Vec<_>>()}],
        "nodes": nodes,
        "meshes": meshes,
    });
    if !materials.is_empty() {
        root["materials"] = json!(materials);
    }
    if !images.is_empty() {
        root["images"] = json!(images);
        root["textures"] = json!(textures);
        root["samplers"] = json!([{"magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497}]);
    }
    if frames > 0 {
        let fps = if o.fps > 0.0 { o.fps } else { 30.0 };
        let times: Vec<f32> = (0..frames).map(|k| k as f32 / fps).collect();
        let input = buf.floats(&times, Some((0.0, times[frames - 1])));
        let mut identity = vec![0.0f32; frames * frames];
        for k in 0..frames {
            identity[k * frames + k] = 1.0;
        }
        let output = buf.floats(&identity, None);
        let mut samplers = Vec::new();
        let mut channels = Vec::new();
        for node in weight_nodes {
            samplers.push(json!({"input": input, "output": output, "interpolation": "LINEAR"}));
            channels.push(json!({"sampler": samplers.len() - 1, "target": {"node": node, "path": "weights"}}));
        }
        root["animations"] = json!([{"name": "cache", "samplers": samplers, "channels": channels}]);
    }
    root["bufferViews"] = json!(buf.views);
    root["accessors"] = json!(buf.accessors);
    let bin_name = format!("{}.bin", o.stem);
    if glb {
        root["buffers"] = json!([{"byteLength": buf.bytes.len()}]);
    } else {
        root["buffers"] = json!([{"byteLength": buf.bytes.len(), "uri": bin_name}]);
    }
    let mut text = serde_json::to_vec(&root).map_err(|e| e.to_string())?;
    if !glb {
        return Ok(vec![File { name: format!("{}.gltf", o.stem), bytes: text }, File { name: bin_name, bytes: buf.bytes }]);
    }
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    let mut bin = buf.bytes;
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let total = 12 + 8 + text.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4E4F_534Au32.to_le_bytes());
    out.extend_from_slice(&text);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x004E_4942u32.to_le_bytes());
    out.extend_from_slice(&bin);
    Ok(vec![File { name: format!("{}.glb", o.stem), bytes: out }])
}
