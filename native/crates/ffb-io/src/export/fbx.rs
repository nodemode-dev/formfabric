use std::fmt::Write;

use super::{Animation, File, Options, Scene};

const TICKS: f64 = 46_186_158_000.0;

enum P {
    I(i32),
    L(i64),
    D(f64),
    S(String),
    Obj(&'static str, String),
    Ints(Vec<i32>),
    Doubles(Vec<f64>),
    Longs(Vec<i64>),
    Floats(Vec<f32>),
}

struct N {
    name: &'static str,
    props: Vec<P>,
    kids: Vec<N>,
}

fn n(name: &'static str, props: Vec<P>, kids: Vec<N>) -> N {
    N { name, props, kids }
}

fn s(text: &str) -> P {
    P::S(text.to_string())
}

fn prop(name: &str, kind: &str, label: &str, flags: &str, values: Vec<P>) -> N {
    let mut props = vec![s(name), s(kind), s(label), s(flags)];
    props.extend(values);
    n("P", props, Vec::new())
}

fn props70(list: Vec<N>) -> N {
    n("Properties70", Vec::new(), list)
}

fn ascii_string(text: &str) -> String {
    text.replace('"', "&quot;")
}

fn write_ascii(out: &mut String, node: &N, depth: usize) {
    let pad = "\t".repeat(depth);
    let _ = write!(out, "{pad}{}: ", node.name);
    let array = node.props.len() == 1 && matches!(node.props[0], P::Ints(_) | P::Doubles(_) | P::Longs(_) | P::Floats(_));
    if array {
        let (count, body) = match &node.props[0] {
            P::Ints(v) => (v.len(), v.iter().map(i32::to_string).collect::<Vec<_>>().join(",")),
            P::Longs(v) => (v.len(), v.iter().map(i64::to_string).collect::<Vec<_>>().join(",")),
            P::Doubles(v) => (v.len(), v.iter().map(f64::to_string).collect::<Vec<_>>().join(",")),
            P::Floats(v) => (v.len(), v.iter().map(f32::to_string).collect::<Vec<_>>().join(",")),
            _ => (0, String::new()),
        };
        let _ = writeln!(out, "*{count} {{\n{pad}\ta: {body}\n{pad}}}");
        return;
    }
    let texts: Vec<String> = node
        .props
        .iter()
        .map(|p| match p {
            P::I(v) => v.to_string(),
            P::L(v) => v.to_string(),
            P::D(v) => format!("{v}"),
            P::S(v) => format!("\"{}\"", ascii_string(v)),
            P::Obj(class, name) => format!("\"{}::{}\"", class, ascii_string(name)),
            _ => String::new(),
        })
        .collect();
    out.push_str(&texts.join(", "));
    if node.kids.is_empty() && !node.props.is_empty() {
        out.push('\n');
        return;
    }
    out.push_str(" {\n");
    for k in &node.kids {
        write_ascii(out, k, depth + 1);
    }
    let _ = writeln!(out, "{pad}}}");
}

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn array_bytes(out: &mut Vec<u8>, code: u8, count: usize, element: usize, fill: impl FnOnce(&mut Vec<u8>)) {
    out.push(code);
    put_u32(out, count as u32);
    put_u32(out, 0);
    put_u32(out, (count * element) as u32);
    fill(out);
}

fn write_binary(out: &mut Vec<u8>, node: &N) {
    let start = out.len();
    put_u32(out, 0);
    put_u32(out, node.props.len() as u32);
    put_u32(out, 0);
    out.push(node.name.len() as u8);
    out.extend_from_slice(node.name.as_bytes());
    let props_at = out.len();
    for p in &node.props {
        match p {
            P::I(v) => {
                out.push(b'I');
                out.extend_from_slice(&v.to_le_bytes());
            }
            P::L(v) => {
                out.push(b'L');
                out.extend_from_slice(&v.to_le_bytes());
            }
            P::D(v) => {
                out.push(b'D');
                out.extend_from_slice(&v.to_le_bytes());
            }
            P::S(v) => {
                out.push(b'S');
                put_u32(out, v.len() as u32);
                out.extend_from_slice(v.as_bytes());
            }
            P::Obj(class, name) => {
                let text = format!("{name}\u{0}\u{1}{class}");
                out.push(b'S');
                put_u32(out, text.len() as u32);
                out.extend_from_slice(text.as_bytes());
            }
            P::Ints(v) => array_bytes(out, b'i', v.len(), 4, |o| v.iter().for_each(|x| o.extend_from_slice(&x.to_le_bytes()))),
            P::Longs(v) => array_bytes(out, b'l', v.len(), 8, |o| v.iter().for_each(|x| o.extend_from_slice(&x.to_le_bytes()))),
            P::Doubles(v) => array_bytes(out, b'd', v.len(), 8, |o| v.iter().for_each(|x| o.extend_from_slice(&x.to_le_bytes()))),
            P::Floats(v) => array_bytes(out, b'f', v.len(), 4, |o| v.iter().for_each(|x| o.extend_from_slice(&x.to_le_bytes()))),
        }
    }
    let props_len = (out.len() - props_at) as u32;
    out[start + 8..start + 12].copy_from_slice(&props_len.to_le_bytes());
    if !node.kids.is_empty() {
        for k in &node.kids {
            write_binary(out, k);
        }
        out.extend_from_slice(&[0u8; 13]);
    }
    let end = out.len() as u32;
    out[start..start + 4].copy_from_slice(&end.to_le_bytes());
}

fn finish_binary(roots: &[N]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"Kaydara FBX Binary  \0\x1a\0");
    put_u32(&mut out, 7400);
    for r in roots {
        write_binary(&mut out, r);
    }
    out.extend_from_slice(&[0u8; 13]);
    out.extend_from_slice(&[0xfa, 0xbc, 0xab, 0x09, 0xd0, 0xc8, 0xd4, 0x66, 0xb1, 0x76, 0xfb, 0x83, 0x1c, 0xf7, 0x26, 0x7e]);
    out.extend_from_slice(&[0u8; 4]);
    let pad = (16 - out.len() % 16) % 16;
    out.extend(std::iter::repeat_n(0u8, pad));
    put_u32(&mut out, 7400);
    out.extend_from_slice(&[0u8; 120]);
    out.extend_from_slice(&[0xf8, 0x5a, 0x8c, 0x6a, 0xde, 0xf5, 0xd9, 0x7e, 0xec, 0xe9, 0x0c, 0xe3, 0x75, 0x8f, 0x29, 0x0b]);
    out
}

fn flat(v: &[[f32; 3]]) -> Vec<f64> {
    v.iter().flat_map(|p| p.iter().map(|x| f64::from(*x))).collect()
}

struct Ids(i64);

impl Ids {
    fn next(&mut self) -> i64 {
        self.0 += 1;
        self.0
    }
}

pub fn write(scene: &Scene, animation: Option<&Animation>, o: &Options) -> Result<Vec<File>, String> {
    let mut ids = Ids(1_000_000);
    let mut objects: Vec<N> = Vec::new();
    let mut links: Vec<N> = Vec::new();
    let link = |kind: &str, child: i64, parent: i64, extra: Option<&str>| -> N {
        let mut props = vec![s(kind), P::L(child), P::L(parent)];
        if let Some(e) = extra {
            props.push(s(e));
        }
        n("C", props, Vec::new())
    };
    let mut material_ids = Vec::new();
    let mut texture_count = 0;
    if o.materials {
        for m in &scene.materials {
            let id = ids.next();
            material_ids.push(id);
            let mut list = vec![
                prop("DiffuseColor", "Color", "", "A", vec![P::D(f64::from(m.color[0])), P::D(f64::from(m.color[1])), P::D(f64::from(m.color[2]))]),
                prop("Opacity", "double", "Number", "", vec![P::D(f64::from(m.opacity * m.color[3]))]),
                prop("Shininess", "double", "Number", "", vec![P::D(f64::from((1.0 - m.roughness).clamp(0.0, 1.0) * 100.0))]),
                prop("Metallic", "double", "Number", "", vec![P::D(f64::from(m.metalness))]),
                prop("Roughness", "double", "Number", "", vec![P::D(f64::from(m.roughness))]),
            ];
            list.push(prop("AmbientColor", "Color", "", "A", vec![P::D(0.0), P::D(0.0), P::D(0.0)]));
            objects.push(n("Material", vec![P::L(id), P::Obj("Material", m.name.clone()), s("")], vec![n("Version", vec![P::I(102)], vec![]), n("ShadingModel", vec![s("phong")], vec![]), n("MultiLayer", vec![P::I(0)], vec![]), props70(list)]));
            if let (Some(path), true) = (&m.base_color_map, o.uv) {
                let tex = ids.next();
                let video = ids.next();
                texture_count += 1;
                let file = path.replace('\\', "/");
                let leaf = file.rsplit('/').next().unwrap_or(&file).to_string();
                objects.push(n(
                    "Texture",
                    vec![P::L(tex), P::Obj("Texture", format!("{}_color", m.name)), s("")],
                    vec![
                        n("Type", vec![s("TextureVideoClip")], vec![]),
                        n("Version", vec![P::I(202)], vec![]),
                        n("TextureName", vec![P::Obj("Texture", format!("{}_color", m.name))], vec![]),
                        n("Media", vec![P::Obj("Video", format!("{}_color", m.name))], vec![]),
                        n("FileName", vec![P::S(file.clone())], vec![]),
                        n("RelativeFilename", vec![P::S(leaf.clone())], vec![]),
                        n("ModelUVTranslation", vec![P::D(0.0), P::D(0.0)], vec![]),
                        n("ModelUVScaling", vec![P::D(1.0), P::D(1.0)], vec![]),
                        n("Texture_Alpha_Source", vec![s("None")], vec![]),
                        n("Cropping", vec![P::I(0), P::I(0), P::I(0), P::I(0)], vec![]),
                    ],
                ));
                objects.push(n("Video", vec![P::L(video), P::Obj("Video", format!("{}_color", m.name)), s("Clip")], vec![n("Type", vec![s("Clip")], vec![]), n("FileName", vec![P::S(file)], vec![]), n("RelativeFilename", vec![P::S(leaf)], vec![])]));
                links.push(link("OO", video, tex, None));
                links.push(link("OP", tex, id, Some("DiffuseColor")));
            }
        }
    }
    let frames = animation.map_or(0, |a| a.frames.len());
    let fps = if o.fps > 0.0 { f64::from(o.fps) } else { 30.0 };
    let mut at = 0usize;
    let mut curve_nodes: Vec<(i64, i64)> = Vec::new();
    let mut anim_objects: Vec<N> = Vec::new();
    let layer_id = if frames > 0 { ids.next() } else { 0 };
    let stack_id = if frames > 0 { ids.next() } else { 0 };
    let mut geometry_count = 0;
    for obj in &scene.objects {
        let nv = obj.positions.len();
        let geom = ids.next();
        let model = ids.next();
        geometry_count += 1;
        let mut poly = Vec::with_capacity(obj.indices.len());
        for t in obj.indices.as_chunks::<3>().0 {
            poly.push(t[0] as i32);
            poly.push(t[1] as i32);
            poly.push(!(t[2] as i32));
        }
        for q in &obj.quads {
            poly.extend_from_slice(&[q[0] as i32, q[1] as i32, q[2] as i32, !(q[3] as i32)]);
        }
        let corners: Vec<u32> = obj.indices.iter().copied().chain(obj.quads.iter().flatten().copied()).collect();
        let corner_normals: Vec<f64> = corners.iter().flat_map(|i| obj.normals[*i as usize].iter().map(|x| f64::from(*x))).collect();
        let mut kids = vec![
            n("Properties70", vec![], vec![]),
            n("GeometryVersion", vec![P::I(124)], vec![]),
            n("Vertices", vec![P::Doubles(flat(&obj.positions))], vec![]),
            n("PolygonVertexIndex", vec![P::Ints(poly)], vec![]),
            n(
                "LayerElementNormal",
                vec![P::I(0)],
                vec![
                    n("Version", vec![P::I(101)], vec![]),
                    n("Name", vec![s("")], vec![]),
                    n("MappingInformationType", vec![s("ByPolygonVertex")], vec![]),
                    n("ReferenceInformationType", vec![s("Direct")], vec![]),
                    n("Normals", vec![P::Doubles(corner_normals)], vec![]),
                ],
            ),
        ];
        let mut layer = vec![n("Version", vec![P::I(100)], vec![]), n("LayerElement", vec![], vec![n("Type", vec![s("LayerElementNormal")], vec![]), n("TypedIndex", vec![P::I(0)], vec![])])];
        if o.uv {
            let uv: Vec<f64> = obj.uvs.iter().flat_map(|u| [f64::from(u[0]), f64::from(u[1])]).collect();
            kids.push(n(
                "LayerElementUV",
                vec![P::I(0)],
                vec![
                    n("Version", vec![P::I(101)], vec![]),
                    n("Name", vec![s("UVMap")], vec![]),
                    n("MappingInformationType", vec![s("ByPolygonVertex")], vec![]),
                    n("ReferenceInformationType", vec![s("IndexToDirect")], vec![]),
                    n("UV", vec![P::Doubles(uv)], vec![]),
                    n("UVIndex", vec![P::Ints(if obj.uv_ix.is_empty() { corners.iter().map(|i| *i as i32).collect() } else { obj.uv_ix.iter().map(|i| *i as i32).collect() })], vec![]),
                ],
            ));
            layer.push(n("LayerElement", vec![], vec![n("Type", vec![s("LayerElementUV")], vec![]), n("TypedIndex", vec![P::I(0)], vec![])]));
        }
        let mut used: Vec<usize> = Vec::new();
        if o.materials {
            for g in &obj.groups {
                if g.material < material_ids.len() && !used.contains(&g.material) {
                    used.push(g.material);
                }
            }
            if !used.is_empty() {
                let mut per_face = vec![0i32; obj.indices.len() / 3 + obj.quads.len()];
                for g in &obj.groups {
                    let local = used.iter().position(|u| *u == g.material).unwrap_or(0) as i32;
                    let end = (g.first + g.count).min(per_face.len());
                    per_face[g.first.min(end)..end].fill(local);
                }
                kids.push(n(
                    "LayerElementMaterial",
                    vec![P::I(0)],
                    vec![
                        n("Version", vec![P::I(101)], vec![]),
                        n("Name", vec![s("")], vec![]),
                        n("MappingInformationType", vec![s(if used.len() == 1 { "AllSame" } else { "ByPolygon" })], vec![]),
                        n("ReferenceInformationType", vec![s("IndexToDirect")], vec![]),
                        n("Materials", vec![P::Ints(if used.len() == 1 { vec![0] } else { per_face })], vec![]),
                    ],
                ));
                layer.push(n("LayerElement", vec![], vec![n("Type", vec![s("LayerElementMaterial")], vec![]), n("TypedIndex", vec![P::I(0)], vec![])]));
            }
        }
        kids.push(n("Layer", vec![P::I(0)], layer));
        objects.push(n("Geometry", vec![P::L(geom), P::Obj("Geometry", obj.name.clone()), s("Mesh")], kids));
        objects.push(n(
            "Model",
            vec![P::L(model), P::Obj("Model", obj.name.clone()), s("Mesh")],
            vec![n("Version", vec![P::I(232)], vec![]), props70(vec![prop("DefaultAttributeIndex", "int", "Integer", "", vec![P::I(0)])]), n("MultiLayer", vec![P::I(0)], vec![]), n("MultiTakeMode", vec![P::I(0)], vec![]), n("Shading", vec![s("T")], vec![]), n("Culling", vec![s("CullingOff")], vec![])],
        ));
        links.push(link("OO", model, 0, None));
        links.push(link("OO", geom, model, None));
        for u in &used {
            links.push(link("OO", material_ids[*u], model, None));
        }
        if let Some(a) = animation {
            let deformer = ids.next();
            anim_objects.push(n("Deformer", vec![P::L(deformer), P::Obj("Deformer", format!("{}_shapes", obj.name)), s("BlendShape")], vec![n("Version", vec![P::I(100)], vec![])]));
            links.push(link("OO", deformer, geom, None));
            let all: Vec<i32> = (0..nv as i32).collect();
            for (k, f) in a.frames.iter().enumerate() {
                let shape = ids.next();
                let channel = ids.next();
                let delta: Vec<[f32; 3]> = f[at..at + nv].iter().zip(&obj.positions).map(|(p, b)| [p[0] - b[0], p[1] - b[1], p[2] - b[2]]).collect();
                let name = format!("{}_f{:04}", obj.name, o.start_frame as usize + k);
                anim_objects.push(n("Geometry", vec![P::L(shape), P::Obj("Geometry", name.clone()), s("Shape")], vec![n("Version", vec![P::I(100)], vec![]), n("Indexes", vec![P::Ints(all.clone())], vec![]), n("Vertices", vec![P::Doubles(flat(&delta))], vec![])]));
                anim_objects.push(n("Deformer", vec![P::L(channel), P::Obj("SubDeformer", name), s("BlendShapeChannel")], vec![n("Version", vec![P::I(100)], vec![]), n("DeformPercent", vec![P::D(0.0)], vec![]), n("FullWeights", vec![P::Doubles(vec![100.0])], vec![])]));
                links.push(link("OO", shape, channel, None));
                links.push(link("OO", channel, deformer, None));
                let node = ids.next();
                let curve = ids.next();
                anim_objects.push(n("AnimationCurveNode", vec![P::L(node), P::Obj("AnimCurveNode", "DeformPercent".into()), s("")], vec![props70(vec![prop("d|DeformPercent", "Number", "", "A", vec![P::D(0.0)])])]));
                let mut times = Vec::new();
                let mut values = Vec::new();
                if k > 0 {
                    times.push(((k - 1) as f64 / fps * TICKS) as i64);
                    values.push(0.0f32);
                }
                times.push((k as f64 / fps * TICKS) as i64);
                values.push(100.0);
                if k + 1 < frames {
                    times.push(((k + 1) as f64 / fps * TICKS) as i64);
                    values.push(0.0);
                }
                let count = times.len() as i32;
                anim_objects.push(n(
                    "AnimationCurve",
                    vec![P::L(curve), P::Obj("AnimCurve", String::new()), s("")],
                    vec![
                        n("Default", vec![P::D(0.0)], vec![]),
                        n("KeyVer", vec![P::I(4008)], vec![]),
                        n("KeyTime", vec![P::Longs(times)], vec![]),
                        n("KeyValueFloat", vec![P::Floats(values)], vec![]),
                        n("KeyAttrFlags", vec![P::Ints(vec![4])], vec![]),
                        n("KeyAttrDataFloat", vec![P::Floats(vec![0.0; 4])], vec![]),
                        n("KeyAttrRefCount", vec![P::Ints(vec![count])], vec![]),
                    ],
                ));
                links.push(link("OP", curve, node, Some("d|DeformPercent")));
                links.push(link("OP", node, channel, Some("DeformPercent")));
                curve_nodes.push((node, layer_id));
            }
        }
        at += nv;
    }
    if frames > 0 {
        let last = ((frames - 1) as f64 / fps * TICKS) as i64;
        anim_objects.push(n("AnimationStack", vec![P::L(stack_id), P::Obj("AnimStack", "cache".into()), s("")], vec![props70(vec![prop("LocalStart", "KTime", "Time", "", vec![P::L(0)]), prop("LocalStop", "KTime", "Time", "", vec![P::L(last)]), prop("ReferenceStart", "KTime", "Time", "", vec![P::L(0)]), prop("ReferenceStop", "KTime", "Time", "", vec![P::L(last)])])]));
        anim_objects.push(n("AnimationLayer", vec![P::L(layer_id), P::Obj("AnimLayer", "BaseLayer".into()), s("")], vec![]));
        links.push(link("OO", layer_id, stack_id, None));
        for (node, layer) in &curve_nodes {
            links.push(link("OO", *node, *layer, None));
        }
    }
    objects.extend(anim_objects);
    let (up, up_sign, front, front_sign) = if o.z_up { (2, 1, 1, -1) } else { (1, 1, 2, 1) };
    let counts = [("GlobalSettings", 1), ("Model", scene.objects.len() as i32), ("Geometry", geometry_count), ("Material", material_ids.len() as i32), ("Texture", texture_count)];
    let mut definitions = vec![n("Version", vec![P::I(100)], vec![]), n("Count", vec![P::I(counts.iter().map(|c| c.1).sum::<i32>().max(1))], vec![])];
    for (kind, count) in counts {
        if count > 0 {
            definitions.push(n("ObjectType", vec![s(kind)], vec![n("Count", vec![P::I(count)], vec![])]));
        }
    }
    let top = vec![
        n("FBXHeaderExtension", vec![], vec![n("FBXHeaderVersion", vec![P::I(1003)], vec![]), n("FBXVersion", vec![P::I(7400)], vec![]), n("Creator", vec![s("formfabric")], vec![])]),
        n(
            "GlobalSettings",
            vec![],
            vec![
                n("Version", vec![P::I(1000)], vec![]),
                props70(vec![
                    prop("UpAxis", "int", "Integer", "", vec![P::I(up)]),
                    prop("UpAxisSign", "int", "Integer", "", vec![P::I(up_sign)]),
                    prop("FrontAxis", "int", "Integer", "", vec![P::I(front)]),
                    prop("FrontAxisSign", "int", "Integer", "", vec![P::I(front_sign)]),
                    prop("CoordAxis", "int", "Integer", "", vec![P::I(0)]),
                    prop("CoordAxisSign", "int", "Integer", "", vec![P::I(1)]),
                    prop("UnitScaleFactor", "double", "Number", "", vec![P::D(o.meters_per_unit * 100.0)]),
                    prop("OriginalUnitScaleFactor", "double", "Number", "", vec![P::D(o.meters_per_unit * 100.0)]),
                    prop("TimeMode", "enum", "", "", vec![P::I(14)]),
                    prop("CustomFrameRate", "double", "Number", "", vec![P::D(fps)]),
                ]),
            ],
        ),
        n("Documents", vec![], vec![n("Count", vec![P::I(1)], vec![]), n("Document", vec![P::L(1), s("Scene"), s("Scene")], vec![n("RootNode", vec![P::L(0)], vec![])])]),
        n("References", vec![], vec![]),
        n("Definitions", vec![], definitions),
        n("Objects", vec![], objects),
        n("Connections", vec![], links),
        n("Takes", vec![], vec![n("Current", vec![s("")], vec![])]),
    ];
    let bytes = if o.binary_fbx {
        finish_binary(&top)
    } else {
        let mut out = String::from("; FBX 7.4.0 project file\n; Created by formfabric\n\n");
        for t in &top {
            write_ascii(&mut out, t, 0);
        }
        out.into_bytes()
    };
    Ok(vec![File { name: format!("{}.fbx", o.stem), bytes }])
}
