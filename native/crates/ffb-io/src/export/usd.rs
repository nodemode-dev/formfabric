use std::fmt::Write;

use super::{Animation, File, Options, Scene, V3};

fn ident(name: &str, prefix: &str, index: usize) -> String {
    let body: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
    format!("{prefix}{index}_{body}")
}

fn tuples3(v: &[V3]) -> String {
    let mut s = String::with_capacity(v.len() * 24);
    s.push('[');
    for (i, p) in v.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "({}, {}, {})", p[0], p[1], p[2]);
    }
    s.push(']');
    s
}

fn ints(v: impl Iterator<Item = u32>) -> String {
    let mut s = String::from("[");
    for (i, x) in v.enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{x}");
    }
    s.push(']');
    s
}

fn path_literal(p: &str) -> String {
    format!("@{}@", p.replace('\\', "/").replace('@', "_"))
}

pub fn write(scene: &Scene, animation: Option<&Animation>, o: &Options) -> Result<Vec<File>, String> {
    let frames = animation.map_or(0, |a| a.frames.len());
    let fps = if o.fps > 0.0 { o.fps } else { 30.0 };
    let mut s = String::with_capacity(scene.vertex_count() * 80);
    s.push_str("#usda 1.0\n(\n    defaultPrim = \"Garment\"\n    doc = \"formfabric export\"\n");
    let _ = writeln!(s, "    metersPerUnit = {}", o.meters_per_unit);
    let _ = writeln!(s, "    upAxis = \"{}\"", if o.z_up { "Z" } else { "Y" });
    if frames > 0 {
        let _ = writeln!(s, "    startTimeCode = {}\n    endTimeCode = {}\n    timeCodesPerSecond = {}", o.start_frame, o.start_frame as usize + frames - 1, fps);
    }
    s.push_str(")\n\ndef Xform \"Garment\"\n{\n");
    let mat_path = |i: usize| format!("/Garment/Materials/{}", ident(&scene.materials[i].name, "M", i));
    if o.materials {
        s.push_str("    def Scope \"Materials\"\n    {\n");
        for (i, m) in scene.materials.iter().enumerate() {
            let name = ident(&m.name, "M", i);
            let base = format!("/Garment/Materials/{name}");
            let _ = writeln!(s, "        def Material \"{name}\"\n        {{");
            let _ = writeln!(s, "            token outputs:surface.connect = <{base}/Surface.outputs:surface>");
            s.push_str("            def Shader \"Surface\"\n            {\n                uniform token info:id = \"UsdPreviewSurface\"\n");
            match (&m.base_color_map, o.uv) {
                (Some(_), true) => {
                    let _ = writeln!(s, "                color3f inputs:diffuseColor.connect = <{base}/Tex.outputs:rgb>");
                }
                _ => {
                    let _ = writeln!(s, "                color3f inputs:diffuseColor = ({}, {}, {})", m.color[0], m.color[1], m.color[2]);
                }
            }
            let _ = writeln!(s, "                float inputs:roughness = {}\n                float inputs:metallic = {}\n                float inputs:opacity = {}", m.roughness, m.metalness, m.opacity * m.color[3]);
            s.push_str("                token outputs:surface\n            }\n");
            if let (Some(path), true) = (&m.base_color_map, o.uv) {
                s.push_str("            def Shader \"StReader\"\n            {\n                uniform token info:id = \"UsdPrimvarReader_float2\"\n                token inputs:varname = \"st\"\n                float2 outputs:result\n            }\n");
                let _ = writeln!(s, "            def Shader \"Tex\"\n            {{\n                uniform token info:id = \"UsdUVTexture\"\n                asset inputs:file = {}\n                float2 inputs:st.connect = <{base}/StReader.outputs:result>\n                token inputs:wrapS = \"repeat\"\n                token inputs:wrapT = \"repeat\"\n                float3 outputs:rgb\n            }}", path_literal(path));
            }
            s.push_str("        }\n");
        }
        s.push_str("    }\n");
    }
    let mut at = 0usize;
    for (oi, obj) in scene.objects.iter().enumerate() {
        let n = obj.positions.len();
        let name = ident(&obj.name, "O", oi);
        let single = obj.groups.len() == 1 && o.materials && obj.groups[0].material < scene.materials.len();
        let _ = writeln!(s, "    def Mesh \"{name}\" (\n        prepend apiSchemas = [\"MaterialBindingAPI\"]\n    )\n    {{");
        let tris = obj.indices.len() / 3;
        let _ = writeln!(s, "        int[] faceVertexCounts = {}", ints(std::iter::repeat_n(3u32, tris).chain(std::iter::repeat_n(4u32, obj.quads.len()))));
        let _ = writeln!(s, "        int[] faceVertexIndices = {}", ints(obj.indices.iter().copied().chain(obj.quads.iter().flatten().copied())));
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        let mut grow = |p: &V3| {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        };
        match animation {
            Some(a) => {
                for f in &a.frames {
                    f[at..at + n].iter().for_each(&mut grow);
                }
                s.push_str("        point3f[] points.timeSamples = {\n");
                for (k, f) in a.frames.iter().enumerate() {
                    let _ = writeln!(s, "            {}: {},", o.start_frame as usize + k, tuples3(&f[at..at + n]));
                }
                s.push_str("        }\n");
            }
            None => {
                obj.positions.iter().for_each(&mut grow);
                let _ = writeln!(s, "        point3f[] points = {}", tuples3(&obj.positions));
            }
        }
        let _ = writeln!(s, "        float3[] extent = [({}, {}, {}), ({}, {}, {})]", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        let _ = writeln!(s, "        normal3f[] normals = {} (\n            interpolation = \"vertex\"\n        )", tuples3(&obj.normals));
        if o.uv {
            let mut t = String::from("[");
            for (i, p) in obj.uvs.iter().enumerate() {
                if i > 0 {
                    t.push_str(", ");
                }
                let _ = write!(t, "({}, {})", p[0], p[1]);
            }
            t.push(']');
            if obj.uv_ix.is_empty() {
                let _ = writeln!(s, "        texCoord2f[] primvars:st = {t} (\n            interpolation = \"vertex\"\n        )");
            } else {
                let _ = writeln!(s, "        texCoord2f[] primvars:st = {t} (\n            interpolation = \"faceVarying\"\n        )");
                let _ = writeln!(s, "        int[] primvars:st:indices = {}", ints(obj.uv_ix.iter().copied()));
            }
        }
        s.push_str("        uniform token subdivisionScheme = \"none\"\n");
        if single {
            let _ = writeln!(s, "        rel material:binding = <{}>", mat_path(obj.groups[0].material));
        } else if o.materials {
            for (gi, g) in obj.groups.iter().enumerate() {
                if g.material >= scene.materials.len() {
                    continue;
                }
                let _ = writeln!(s, "        def GeomSubset \"G{gi}\"\n        (\n            prepend apiSchemas = [\"MaterialBindingAPI\"]\n        )\n        {{\n            uniform token elementType = \"face\"\n            uniform token familyName = \"materialBind\"\n            int[] indices = {}\n            rel material:binding = <{}>\n        }}", ints((g.first as u32)..((g.first + g.count) as u32)), mat_path(g.material));
            }
        }
        s.push_str("    }\n");
        at += n;
    }
    s.push_str("}\n");
    Ok(vec![File { name: format!("{}.usda", o.stem), bytes: s.into_bytes() }])
}
