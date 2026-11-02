use super::{vertex_normals, Animation, File, Object, Options, Scene, V3};
use crate::abc::archive::Archive;
use crate::abc::geom::{xform_with_mesh, Mesh};

fn clean(name: &str, taken: &mut Vec<String>) -> String {
    let mut base: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' { c } else { '_' }).collect();
    if base.is_empty() {
        base = "object".into();
    }
    let mut out = base.clone();
    let mut k = 2;
    while taken.contains(&out) {
        out = format!("{base}_{k}");
        k += 1;
    }
    taken.push(out.clone());
    out
}

struct Faces {
    counts: Vec<i32>,
    indices: Vec<i32>,
    corners: Vec<u32>,
    source: Vec<usize>,
    triangles: Vec<u32>,
}

fn faces(o: &Object) -> Faces {
    let tris = o.indices.len() / 3;
    let mut f = Faces { counts: Vec::with_capacity(tris + o.quads.len()), indices: Vec::with_capacity(o.indices.len() + o.quads.len() * 4), corners: Vec::new(), source: Vec::new(), triangles: o.indices.clone() };
    for (k, t) in o.indices.as_chunks::<3>().0.iter().enumerate() {
        f.counts.push(3);
        f.corners.extend([t[2], t[1], t[0]]);
        f.source.extend([3 * k + 2, 3 * k + 1, 3 * k]);
    }
    let base = o.indices.len();
    for (k, q) in o.quads.iter().enumerate() {
        f.counts.push(4);
        f.corners.extend([q[3], q[2], q[1], q[0]]);
        let c = base + 4 * k;
        f.source.extend([c + 3, c + 2, c + 1, c]);
        f.triangles.extend([q[0], q[1], q[2], q[0], q[2], q[3]]);
    }
    f.indices = f.corners.iter().map(|i| *i as i32).collect();
    f
}

pub fn write(scene: &Scene, animation: Option<&Animation>, o: &Options) -> Result<Vec<File>, String> {
    let fps = if o.fps > 0.0 { f64::from(o.fps) } else { 30.0 };
    let frames = animation.map_or(0, |a| a.frames.len());
    let animated = frames >= 2;
    let mut archive = Archive::new("formfabric");
    let ts = if animated { archive.add_uniform_sampling(1.0 / fps, 0.0) } else { 0 };
    let mut taken = Vec::new();
    let mut at = 0usize;
    for obj in &scene.objects {
        let n = obj.positions.len();
        let f = faces(obj);
        if f.corners.iter().any(|i| *i as usize >= n) {
            return Err(format!("{} has a face index past its vertices", obj.name));
        }
        let positions: Vec<&[V3]> = match animation {
            Some(a) if frames >= 1 => {
                let take = if animated { frames } else { 1 };
                a.frames[..take].iter().map(|fr| &fr[at..at + n]).collect()
            }
            _ => vec![&obj.positions[..]],
        };
        let uv = if !o.uv {
            None
        } else if obj.uv_ix.is_empty() {
            (obj.uvs.len() == n).then(|| f.corners.iter().map(|i| obj.uvs[*i as usize]).collect::<Vec<_>>())
        } else if obj.uv_ix.len() == f.corners.len() && obj.uv_ix.iter().all(|i| (*i as usize) < obj.uvs.len()) {
            Some(f.source.iter().map(|c| obj.uvs[obj.uv_ix[*c] as usize]).collect::<Vec<_>>())
        } else {
            None
        };
        let normals = if animated {
            Some(positions.iter().map(|p| vertex_normals(p, &f.triangles)).collect::<Vec<_>>())
        } else if obj.normals.len() == n && animation.is_none() {
            Some(vec![obj.normals.clone()])
        } else {
            Some(vec![vertex_normals(positions[0], &f.triangles)])
        };
        let name = clean(&obj.name, &mut taken);
        let mesh = Mesh { name: &name, positions, face_counts: f.counts, face_indices: f.indices, uv, normals, ts };
        archive.root.push(xform_with_mesh(&name, mesh)?);
        at += n;
    }
    let bytes = archive.to_bytes()?;
    Ok(vec![File { name: format!("{}.abc", o.stem), bytes }])
}
