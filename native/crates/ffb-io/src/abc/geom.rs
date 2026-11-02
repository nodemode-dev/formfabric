use super::archive::{Obj, Pod, Prop};

const POLYMESH_OBJECT: &str = "schema=AbcGeom_PolyMesh_v1;schemaBaseType=AbcGeom_GeomBase_v1;schemaObjTitle=AbcGeom_PolyMesh_v1:.geom";
const POLYMESH_COMPOUND: &str = "schema=AbcGeom_PolyMesh_v1;schemaBaseType=AbcGeom_GeomBase_v1";
const XFORM_OBJECT: &str = "schema=AbcGeom_Xform_v3;schemaObjTitle=AbcGeom_Xform_v3:.xform";
const XFORM_COMPOUND: &str = "schema=AbcGeom_Xform_v3";
const BOX: &str = "interpretation=box";
const POINT: &str = "geoScope=vtx;interpretation=point";
const UV: &str = "arrayExtent=1;geoScope=fvr;interpretation=vector;isGeomParam=true;podExtent=2;podName=float32_t;sourceName=UVMap";
const NORMAL: &str = "arrayExtent=1;geoScope=vtx;interpretation=normal;isGeomParam=true;podExtent=3;podName=float32_t";

pub struct Mesh<'a> {
    pub name: &'a str,
    pub positions: Vec<&'a [[f32; 3]]>,
    pub face_counts: Vec<i32>,
    pub face_indices: Vec<i32>,
    pub uv: Option<Vec<[f32; 2]>>,
    pub normals: Option<Vec<Vec<[f32; 3]>>>,
    pub ts: u32,
}

fn f32_bytes<const N: usize>(v: &[[f32; N]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * N * 4);
    for p in v {
        for c in p {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    out
}

fn i32_bytes(v: &[i32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

pub fn bounds(p: &[[f32; 3]]) -> [f64; 6] {
    let mut b = [f64::MAX, f64::MAX, f64::MAX, f64::MIN, f64::MIN, f64::MIN];
    for q in p {
        for k in 0..3 {
            b[k] = b[k].min(f64::from(q[k]));
            b[k + 3] = b[k + 3].max(f64::from(q[k]));
        }
    }
    if p.is_empty() {
        return [0.0; 6];
    }
    b
}

pub fn xform_with_mesh(xform_name: &str, mesh: Mesh) -> Result<Obj, String> {
    let n = mesh.positions.len();
    if n == 0 {
        return Err("a mesh needs at least one sample".into());
    }
    let bnds: Vec<Vec<u8>> = mesh.positions.iter().map(|p| bounds(p).iter().flat_map(|x| x.to_le_bytes()).collect()).collect();
    let p: Vec<Vec<u8>> = mesh.positions.iter().map(|p| f32_bytes(p)).collect();
    let mut geom = vec![
        Prop::scalar(".selfBnds", BOX, Pod::F64, 6, mesh.ts, n, bnds),
        Prop::array("P", POINT, Pod::F32, 3, mesh.ts, n, p),
        Prop::array(".faceIndices", "", Pod::I32, 1, mesh.ts, n, vec![i32_bytes(&mesh.face_indices)]),
        Prop::array(".faceCounts", "", Pod::I32, 1, mesh.ts, n, vec![i32_bytes(&mesh.face_counts)]),
    ];
    if let Some(uv) = &mesh.uv {
        geom.push(Prop::array("uv", UV, Pod::F32, 2, mesh.ts, n, vec![f32_bytes(uv)]));
    }
    if let Some(normals) = &mesh.normals {
        let list: Vec<Vec<u8>> = normals.iter().map(|x| f32_bytes(x)).collect();
        geom.push(Prop::array("N", NORMAL, Pod::F32, 3, mesh.ts, n, list));
    }
    let shape = Obj { name: format!("{}Shape", mesh.name), meta: POLYMESH_OBJECT.into(), props: vec![Prop::compound(".geom", POLYMESH_COMPOUND, geom)], children: Vec::new() };
    let inherits = Prop::scalar(".inherits", "", Pod::Bool, 1, 0, 1, vec![vec![1u8]]);
    Ok(Obj { name: xform_name.into(), meta: XFORM_OBJECT.into(), props: vec![Prop::compound(".xform", XFORM_COMPOUND, vec![inherits])], children: vec![shape] })
}
