struct VState {
    x: vec4<f32>,
    xn: vec4<f32>,
    y: vec4<f32>,
    v: vec4<f32>,
    a: vec4<f32>,
    e: vec4<f32>,
}

struct VInfo {
    m: vec4<f32>,
    pa: vec4<f32>,
    pb: vec4<f32>,
    ext: vec4<f32>,
}

struct SubU {
    gh: vec4<f32>,
    p0: vec4<f32>,
    p1: vec4<f32>,
    tg: vec4<f32>,
    u0: vec4<u32>,
    u1: vec4<u32>,
    u2: vec4<u32>,
    sc0: vec4<f32>,
    sc1: vec4<u32>,
}

struct ColU {
    r: vec4<u32>,
}

@group(0) @binding(0) var<storage, read_write> vs: array<VState>;
@group(0) @binding(1) var<storage, read> vinfo: array<VInfo>;
@group(0) @binding(2) var<storage, read> sctri: array<u32>;
@group(0) @binding(3) var<storage, read> scv: array<vec4<u32>>;
@group(0) @binding(4) var<storage, read> scn: array<u32>;
@group(0) @binding(5) var<storage, read_write> hmeta: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> hent: array<u32>;
@group(0) @binding(7) var<storage, read_write> disp: array<atomic<i32>>;
@group(1) @binding(0) var<uniform> su: SubU;
@group(1) @binding(1) var<uniform> cu: ColU;

const SCALE: f32 = 100000.0;
const MAX_PUSH: f32 = 3.0;
const BLOCK: u32 = 256u;

fn bucket(c: vec3<i32>) -> u32 {
    let k = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u);
    return (k ^ (k >> 13u)) & (su.sc1.x - 1u);
}

fn tri_lo_hi(t: u32) -> array<vec3<i32>, 2> {
    let a = vs[sctri[3u * t]].x.xyz;
    let b = vs[sctri[3u * t + 1u]].x.xyz;
    let c = vs[sctri[3u * t + 2u]].x.xyz;
    let ext = vec3<f32>(su.sc0.y + su.sc0.z);
    let lo = (min(a, min(b, c)) - ext) / su.sc0.w;
    let hi = (max(a, max(b, c)) + ext) / su.sc0.w;
    return array<vec3<i32>, 2>(vec3<i32>(floor(lo)), vec3<i32>(floor(hi)));
}

@compute @workgroup_size(64)
fn sc_zero(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i < su.sc1.x) {
        atomicStore(&hmeta[i], 0u);
    }
}

@compute @workgroup_size(64)
fn sc_count(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = gid.x;
    if (t >= su.sc1.y) {
        return;
    }
    let r = tri_lo_hi(t);
    let dim = r[1] - r[0] + vec3<i32>(1);
    if (any(r[0] < vec3<i32>(-536870912)) || any(r[1] > vec3<i32>(536870912)) || any(dim > vec3<i32>(64)) || dim.x * dim.y * dim.z > 64) {
        return;
    }
    for (var ix = r[0].x; ix <= r[1].x; ix++) {
        for (var iy = r[0].y; iy <= r[1].y; iy++) {
            for (var iz = r[0].z; iz <= r[1].z; iz++) {
                atomicAdd(&hmeta[bucket(vec3<i32>(ix, iy, iz))], 1u);
            }
        }
    }
}

@compute @workgroup_size(64)
fn sc_scan_local(@builtin(global_invocation_id) gid: vec3<u32>) {
    let k = gid.x;
    let nb = su.sc1.x / BLOCK;
    if (k >= nb) {
        return;
    }
    var run = 0u;
    for (var j = 0u; j < BLOCK; j++) {
        let b = k * BLOCK + j;
        let c = atomicLoad(&hmeta[b]);
        atomicStore(&hmeta[su.sc1.x + b], run);
        run = run + c;
    }
    atomicStore(&hmeta[2u * su.sc1.x + k], run);
}

@compute @workgroup_size(1)
fn sc_scan_blocks() {
    let nb = su.sc1.x / BLOCK;
    var run = 0u;
    for (var k = 0u; k < nb; k++) {
        let c = atomicLoad(&hmeta[2u * su.sc1.x + k]);
        atomicStore(&hmeta[2u * su.sc1.x + k], run);
        run = run + c;
    }
}

@compute @workgroup_size(64)
fn sc_scan_add(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b = gid.x;
    if (b >= su.sc1.x) {
        return;
    }
    let add = atomicLoad(&hmeta[2u * su.sc1.x + b / BLOCK]);
    atomicAdd(&hmeta[su.sc1.x + b], add);
}

@compute @workgroup_size(64)
fn sc_fill(@builtin(global_invocation_id) gid: vec3<u32>) {
    let t = gid.x;
    if (t >= su.sc1.y) {
        return;
    }
    let r = tri_lo_hi(t);
    let dim = r[1] - r[0] + vec3<i32>(1);
    if (any(r[0] < vec3<i32>(-536870912)) || any(r[1] > vec3<i32>(536870912)) || any(dim > vec3<i32>(64)) || dim.x * dim.y * dim.z > 64) {
        return;
    }
    for (var ix = r[0].x; ix <= r[1].x; ix++) {
        for (var iy = r[0].y; iy <= r[1].y; iy++) {
            for (var iz = r[0].z; iz <= r[1].z; iz++) {
                let b = bucket(vec3<i32>(ix, iy, iz));
                let pos = atomicLoad(&hmeta[su.sc1.x + b]) + atomicAdd(&hmeta[b], 1u);
                if (pos < su.sc1.z) {
                    hent[pos] = t;
                }
            }
        }
    }
}

fn closest_pt(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if (d1 <= 0.0 && d2 <= 0.0) {
        return a;
    }
    let bp = p - b;
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if (d3 >= 0.0 && d4 <= d3) {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if (vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0) {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if (d6 >= 0.0 && d5 <= d6) {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if (vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0) {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if (va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0) {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    return a + ab * (vb * denom) + ac * (vc * denom);
}

fn bary(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let den = d00 * d11 - d01 * d01;
    if (abs(den) < 1.0e-20) {
        return vec3<f32>(1.0, 0.0, 0.0);
    }
    let v = (d11 * d20 - d01 * d21) / den;
    let w = (d00 * d21 - d01 * d20) / den;
    return vec3<f32>(1.0 - v - w, v, w);
}

fn has(v: u32, a: u32) -> bool {
    let h = scv[v];
    var lo = 0u;
    var hi = h.w;
    while (lo < hi) {
        let mid = (lo + hi) / 2u;
        let q = scn[h.z + mid];
        if (q == a) {
            return true;
        }
        if (q < a) {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    return false;
}

fn linked(v: u32, a: u32) -> bool {
    if (v == a || has(v, a)) {
        return true;
    }
    let h = scv[v];
    for (var k = 0u; k < h.w; k++) {
        if (has(scn[h.z + k], a)) {
            return true;
        }
    }
    return false;
}

fn inv_mass(i: u32) -> f32 {
    let info = vinfo[i];
    if (info.m.z > 0.5) {
        return 0.0;
    }
    return 1.0 / info.m.x;
}

fn push(i: u32, d: vec3<f32>) {
    atomicAdd(&disp[4u * i], i32(round(d.x * SCALE)));
    atomicAdd(&disp[4u * i + 1u], i32(round(d.y * SCALE)));
    atomicAdd(&disp[4u * i + 2u], i32(round(d.z * SCALE)));
    atomicAdd(&disp[4u * i + 3u], 1);
}

@compute @workgroup_size(64)
fn sc_vt(@builtin(global_invocation_id) gid: vec3<u32>) {
    let v = gid.x;
    if (v >= su.u0.x) {
        return;
    }
    let base = su.sc0.x;
    let reach = su.sc0.y;
    let margin = su.sc0.z;
    let xv = vs[v].x.xyz;
    let xnv = vs[v].xn.xyz;
    let wv = inv_mass(v);
    let b = bucket(vec3<i32>(floor(xv / su.sc0.w)));
    let start = atomicLoad(&hmeta[su.sc1.x + b]);
    let cnt = atomicLoad(&hmeta[b]);
    let vd = scv[v];
    let layer_v = bitcast<i32>(vd.x);
    let th_v = bitcast<f32>(vd.y);
    for (var e = start; e < start + cnt && e < su.sc1.z; e++) {
        let t = hent[e];
        let i0 = sctri[3u * t];
        let i1 = sctri[3u * t + 1u];
        let i2 = sctri[3u * t + 2u];
        if (i0 == v || i1 == v || i2 == v) {
            continue;
        }
        let a = vs[i0].x.xyz;
        let bb = vs[i1].x.xyz;
        let c = vs[i2].x.xyz;
        let cp = closest_pt(xv, a, bb, c);
        let dvec = xv - cp;
        let dist = length(dvec);
        if (dist > reach + margin) {
            continue;
        }
        let tnv = cross(bb - a, c - a);
        let tl = length(tnv);
        if (tl < 1.0e-12) {
            continue;
        }
        let tn = tnv / tl;
        if (linked(v, i0) || linked(v, i1) || linked(v, i2)) {
            continue;
        }
        let d0 = scv[i0];
        let dl = layer_v - bitcast<i32>(d0.x);
        var s_req = 1.0;
        if (dl > 0) {
            s_req = 1.0;
        } else if (dl < 0) {
            s_req = -1.0;
        } else {
            let pv = cross(vs[i1].xn.xyz - vs[i0].xn.xyz, vs[i2].xn.xyz - vs[i0].xn.xyz);
            let pl = length(pv);
            var sp = 0.0;
            if (pl > 1.0e-12) {
                sp = dot(xnv - vs[i0].xn.xyz, pv / pl);
            }
            if (sp > 1.0e-9) {
                s_req = 1.0;
            } else if (sp < -1.0e-9 || dot(dvec, tn) < 0.0) {
                s_req = -1.0;
            } else {
                s_req = 1.0;
            }
        }
        let mean = (th_v + bitcast<f32>(d0.y) + bitcast<f32>(scv[i1].y) + bitcast<f32>(scv[i2].y)) * 0.25;
        let sep = max(base, mean) * f32(clamp(abs(dl), 1, 3));
        if (dist > sep + margin) {
            continue;
        }
        let bc = bary(cp, a, bb, c);
        let interior = bc.x > 1.0e-6 && bc.y > 1.0e-6 && bc.z > 1.0e-6;
        let plane = dot(dvec, tn) * s_req;
        var sd = -dist;
        if (interior) {
            sd = plane;
        } else if (plane >= 0.0) {
            sd = dist;
        }
        if (sd >= sep) {
            continue;
        }
        var n = dvec / max(dist, 1.0e-12);
        if (interior || plane < 0.0 || dist < 1.0e-12) {
            n = tn * s_req;
        }
        let w0 = inv_mass(i0);
        let w1 = inv_mass(i1);
        let w2 = inv_mass(i2);
        let wsum = wv + bc.x * bc.x * w0 + bc.y * bc.y * w1 + bc.z * bc.z * w2;
        if (wsum < 1.0e-18) {
            continue;
        }
        let lambda = min(sep - sd, MAX_PUSH * sep) / wsum;
        if (wv > 0.0) {
            push(v, n * (lambda * wv));
        }
        if (w0 > 0.0) {
            push(i0, -n * (lambda * w0 * bc.x));
        }
        if (w1 > 0.0) {
            push(i1, -n * (lambda * w1 * bc.y));
        }
        if (w2 > 0.0) {
            push(i2, -n * (lambda * w2 * bc.z));
        }
    }
}

@compute @workgroup_size(64)
fn sc_apply(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= su.u0.x) {
        return;
    }
    let cnt = atomicExchange(&disp[4u * i + 3u], 0);
    let dx = atomicExchange(&disp[4u * i], 0);
    let dy = atomicExchange(&disp[4u * i + 1u], 0);
    let dz = atomicExchange(&disp[4u * i + 2u], 0);
    if (cnt > 0) {
        let d = vec3<f32>(f32(dx), f32(dy), f32(dz)) / (SCALE * f32(cnt));
        vs[i].x = vec4<f32>(vs[i].x.xyz + d, 0.0);
    }
}
