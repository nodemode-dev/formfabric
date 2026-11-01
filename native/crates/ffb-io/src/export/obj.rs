use std::fmt::Write;

use super::{Animation, File, Options, Scene, V3};

fn mtl(scene: &Scene) -> String {
    let mut s = String::from("# formfabric\n");
    for m in &scene.materials {
        let _ = writeln!(s, "newmtl {}", clean(&m.name));
        let _ = writeln!(s, "Kd {:.5} {:.5} {:.5}", m.color[0], m.color[1], m.color[2]);
        let _ = writeln!(s, "Ka 0 0 0");
        let _ = writeln!(s, "Ks {0:.3} {0:.3} {0:.3}", (1.0 - m.roughness).clamp(0.0, 1.0) * 0.2);
        let _ = writeln!(s, "Ns {:.2}", ((1.0 - m.roughness).clamp(0.0, 1.0) * 200.0).max(1.0));
        let _ = writeln!(s, "d {:.4}", m.opacity * m.color[3]);
        let _ = writeln!(s, "illum 2");
        if let Some(t) = &m.base_color_map {
            let _ = writeln!(s, "map_Kd {t}");
        }
        if let Some(t) = &m.normal_map {
            let _ = writeln!(s, "map_Bump {t}");
        }
    }
    s
}

pub fn clean(name: &str) -> String {
    let c: String = name.chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' { c } else { '_' }).collect();
    if c.is_empty() {
        "unnamed".into()
    } else {
        c
    }
}

fn body(scene: &Scene, positions: Option<&[V3]>, o: &Options) -> String {
    let mut s = String::with_capacity(scene.vertex_count() * 64);
    s.push_str("# formfabric\n");
    if o.materials {
        let _ = writeln!(s, "mtllib {}.mtl", o.stem);
    }
    let uv = o.uv;
    let (mut vb, mut at) = (1usize, 0usize);
    for obj in &scene.objects {
        let _ = writeln!(s, "o {}", clean(&obj.name));
        let n = obj.positions.len();
        let pos = match positions {
            Some(p) => &p[at..at + n],
            None => &obj.positions[..],
        };
        at += n;
        for p in pos {
            let _ = writeln!(s, "v {} {} {}", p[0], p[1], p[2]);
        }
        if uv {
            for t in &obj.uvs {
                let _ = writeln!(s, "vt {} {}", t[0], t[1]);
            }
        }
        let normals = if positions.is_some() { super::vertex_normals(pos, &obj.indices) } else { obj.normals.clone() };
        for nn in &normals {
            let _ = writeln!(s, "vn {} {} {}", nn[0], nn[1], nn[2]);
        }
        for (gi, g) in obj.groups.iter().enumerate() {
            if o.materials {
                if let Some(m) = scene.materials.get(g.material) {
                    let _ = writeln!(s, "usemtl {}", clean(&m.name));
                }
            }
            let _ = writeln!(s, "g {}_{}", clean(&obj.name), g.first);
            for t in obj.indices[g.first * 3..(g.first + g.count) * 3].chunks_exact(3) {
                let f = |i: u32| {
                    let k = vb + i as usize;
                    if uv {
                        format!("{k}/{k}/{k}")
                    } else {
                        format!("{k}//{k}")
                    }
                };
                let _ = writeln!(s, "f {} {} {}", f(t[0]), f(t[1]), f(t[2]));
            }
            if gi == 0 {
                for q in &obj.quads {
                    let f = |i: u32| {
                        let k = vb + i as usize;
                        if uv {
                            format!("{k}/{k}/{k}")
                        } else {
                            format!("{k}//{k}")
                        }
                    };
                    let _ = writeln!(s, "f {} {} {} {}", f(q[0]), f(q[1]), f(q[2]), f(q[3]));
                }
            }
        }
        vb += n;
    }
    s
}

pub fn write(scene: &Scene, animation: Option<&Animation>, o: &Options) -> Result<Vec<File>, String> {
    let mut files = Vec::new();
    match animation {
        None => files.push(File { name: format!("{}.obj", o.stem), bytes: body(scene, None, o).into_bytes() }),
        Some(a) => {
            for (i, frame) in a.frames.iter().enumerate() {
                let name = format!("{}_{:04}.obj", o.stem, o.start_frame as usize + i + 1);
                files.push(File { name, bytes: body(scene, Some(frame), o).into_bytes() });
            }
        }
    }
    if o.materials {
        files.push(File { name: format!("{}.mtl", o.stem), bytes: mtl(scene).into_bytes() });
    }
    Ok(files)
}
