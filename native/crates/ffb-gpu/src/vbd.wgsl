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
@group(0) @binding(2) var<storage, read> elemu: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read> idx: array<u32>;
@group(0) @binding(4) var<storage, read> colu: array<vec4<u32>>;
@group(0) @binding(5) var<storage, read> cidx: array<u32>;
@group(0) @binding(6) var<storage, read_write> outb: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read_write> hs: array<vec4<f32>>;
@group(1) @binding(0) var<uniform> su: SubU;
@group(1) @binding(1) var<uniform> cu: ColU;

const COMPRESSION: f32 = 0.05;
const CONTACT_K: f32 = 1000.0;
const FRICTION_EPS: f32 = 0.01;
const GRAB_K: f32 = 100.0;
const APPROACH_CAP: f32 = 0.5;
const APPROACH_ZONE: f32 = 8.0;
const DEEP_REACH: f32 = 40.0;
const PI: f32 = 3.14159265358979;
const TAU: f32 = 6.28318530717959;

var<private> G: vec3<f32>;
var<private> H: mat3x3<f32>;

fn outer(a: vec3<f32>, b: vec3<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(a * b.x, a * b.y, a * b.z);
}

fn ident() -> mat3x3<f32> {
    return mat3x3<f32>(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0));
}

fn ub(v: u32) -> u32 {
    return v;
}

fn f4(v: vec4<u32>) -> vec4<f32> {
    return bitcast<vec4<f32>>(v);
}

fn cf(i: u32) -> vec4<f32> {
    return f4(colu[i]);
}

struct ST {
    r: f32,
    n: vec3<f32>,
    m0: f32,
    m1: f32,
}

fn stretch(f: vec3<f32>, k: f32, area: f32) -> ST {
    let len = length(f);
    if (len < 1.0e-12) {
        return ST(0.0, vec3<f32>(1.0, 0.0, 0.0), k * area, k * area);
    }
    let n = f / len;
    let eps = len - 1.0;
    var kg = k * eps;
    var kh = k;
    if (su.p1.x > 0.5 && eps > 0.0) {
        let g = su.p1.y;
        kg = k * (eps + g * eps * eps * eps);
        kh = k * (1.0 + 3.0 * g * eps * eps);
    } else if (eps < 0.0) {
        kg = k * COMPRESSION * eps;
        kh = k * COMPRESSION;
    }
    var tf = 0.0;
    if (eps > 0.0) {
        tf = kg / len;
    }
    return ST(kg * area, n, kh * area, tf * area);
}

fn tri_contrib(base: u32, slot: u32, ih: f32) {
    let t0 = elemu[base];
    let al = f4(elemu[base + 1u]);
    let be = f4(elemu[base + 2u]);
    let kk = f4(elemu[base + 3u]);
    let i0 = ub(t0.x);
    let i1 = ub(t0.y);
    let i2 = ub(t0.z);
    let p0 = vs[i0].x.xyz;
    let p1 = vs[i1].x.xyz;
    let p2 = vs[i2].x.xyz;
    let d0 = p0 - vs[i0].xn.xyz;
    let d1 = p1 - vs[i1].xn.xyz;
    let d2 = p2 - vs[i2].xn.xyz;
    let fu = p0 * al.x + p1 * al.y + p2 * al.z;
    let fv = p0 * be.x + p1 * be.y + p2 * be.z;
    let du = d0 * al.x + d1 * al.y + d2 * al.z;
    let dv = d0 * be.x + d1 * be.y + d2 * be.z;
    let area = al.w;
    let r = be.w * ih;
    let a = al[slot];
    let b = be[slot];
    let su_ = stretch(fu, kk.x, area);
    let sv_ = stretch(fv, kk.y, area);
    let s1 = 1.0 + r;
    let sk = kk.z * area;
    let j = fv * a + fu * b;
    var g = su_.n * (a * su_.r) + sv_.n * (b * sv_.r) + j * (sk * dot(fu, fv));
    H = H + outer(su_.n, su_.n) * ((su_.m0 - su_.m1) * a * a * s1);
    H = H + outer(sv_.n, sv_.n) * ((sv_.m0 - sv_.m1) * b * b * s1);
    H = H + ident() * ((su_.m1 * a * a + sv_.m1 * b * b) * s1);
    H = H + outer(j, j) * (sk * s1);
    if (r > 0.0) {
        let q = su_.n * ((su_.m0 - su_.m1) * dot(su_.n, du)) + du * su_.m1;
        let p = sv_.n * ((sv_.m0 - sv_.m1) * dot(sv_.n, dv)) + dv * sv_.m1;
        g = g + (q * a + p * b + j * (sk * (dot(fv, du) + dot(fu, dv)))) * r;
    }
    G = G + g;
}

fn hinge_eval_store(base: u32, hid: u32) {
    let hb = hid * 9u;
    let h0 = elemu[base];
    let h1 = f4(elemu[base + 1u]);
    let h2 = f4(elemu[base + 2u]);
    let q0 = ub(h0.x);
    let q1 = ub(h0.y);
    let q2 = ub(h0.z);
    let q3 = ub(h0.w);
    let p0 = vs[q0].x.xyz;
    let p1 = vs[q1].x.xyz;
    let p2 = vs[q2].x.xyz;
    let p3 = vs[q3].x.xyz;
    let e = p1 - p0;
    let le = length(e);
    let n1v = cross(e, p2 - p0);
    let n2v = cross(p0 - p1, p3 - p1);
    let l1 = dot(n1v, n1v);
    let l2 = dot(n2v, n2v);
    hs[hb + 4u] = vec4<f32>(p0, 0.0);
    hs[hb + 5u] = vec4<f32>(p1, 0.0);
    hs[hb + 6u] = vec4<f32>(p2, 0.0);
    hs[hb + 7u] = vec4<f32>(p3, 0.0);
    if (le < 1.0e-9 || l1 < 1.0e-18 || l2 < 1.0e-18) {
        hs[hb] = vec4<f32>(0.0);
        hs[hb + 1u] = vec4<f32>(0.0);
        hs[hb + 2u] = vec4<f32>(0.0);
        hs[hb + 3u] = vec4<f32>(0.0);
        hs[hb + 8u] = vec4<f32>(0.0);
        return;
    }
    let eh = e / le;
    let s1 = sqrt(l1);
    let s2 = sqrt(l2);
    let n1 = n1v / s1;
    let n2 = n2v / s2;
    let sin_t = dot(cross(n2, n1), eh);
    let cos_t = dot(n1, n2);
    let theta0 = h1.y;
    var d: f32;
    if (theta0 == 0.0 && cos_t > 0.0 && abs(sin_t) < 0.4) {
        let q = sin_t * sin_t;
        d = sin_t * (1.0 + q * (1.0 / 6.0 + q * (3.0 / 40.0 + q * (5.0 / 112.0 + q * 35.0 / 1152.0))));
    } else {
        d = atan2(sin_t, cos_t) - theta0;
    }
    if (d > PI) {
        d = d - TAU;
    }
    if (d < -PI) {
        d = d + TAU;
    }
    let g2 = n1v * (le / l1);
    let g3 = n2v * (le / l2);
    let a1 = dot(p2 - p0, e) / (le * le);
    let a2 = dot(p3 - p0, e) / (le * le);
    let g0 = -g2 * (1.0 - a1) - g3 * (1.0 - a2);
    let g1 = -g2 * a1 - g3 * a2;
    let lambda = 0.5 * (s1 / le / max(h1.z, 1.0e-9) + s2 / le / max(h1.w, 1.0e-9));
    let compress = 1.0 - lambda;
    var f = 1.0;
    if (h2.x > 0.0 && compress > 0.0) {
        f = 1.0 - (1.0 - h2.y) * min(compress * h2.x / 0.05, 1.0);
    }
    hs[hb] = vec4<f32>(g0, 0.0);
    hs[hb + 1u] = vec4<f32>(g1, 0.0);
    hs[hb + 2u] = vec4<f32>(g2, 0.0);
    hs[hb + 3u] = vec4<f32>(g3, 0.0);
    hs[hb + 8u] = vec4<f32>(h1.x * f, d, 0.0, 0.0);
}

fn hinge_contrib(base: u32, slot: u32, ih: f32, hid: u32) {
    let hb = hid * 9u;
    let cd = hs[hb + 8u];
    if (cd.x == 0.0) {
        return;
    }
    let h0 = elemu[base];
    let h2 = f4(elemu[base + 2u]);
    let q0 = ub(h0.x);
    let q1 = ub(h0.y);
    let q2 = ub(h0.z);
    let q3 = ub(h0.w);
    let g0 = hs[hb].xyz;
    let g1 = hs[hb + 1u].xyz;
    let g2 = hs[hb + 2u].xyz;
    let g3 = hs[hb + 3u].xyz;
    let tl = cd.y + dot(g0, vs[q0].x.xyz - hs[hb + 4u].xyz) + dot(g1, vs[q1].x.xyz - hs[hb + 5u].xyz) + dot(g2, vs[q2].x.xyz - hs[hb + 6u].xyz) + dot(g3, vs[q3].x.xyz - hs[hb + 7u].xyz);
    let ds = dot(g0, vs[q0].x.xyz - vs[q0].xn.xyz) + dot(g1, vs[q1].x.xyz - vs[q1].xn.xyz) + dot(g2, vs[q2].x.xyz - vs[q2].xn.xyz) + dot(g3, vs[q3].x.xyz - vs[q3].xn.xyz);
    var gs = g0;
    if (slot == 1u) {
        gs = g1;
    } else if (slot == 2u) {
        gs = g2;
    } else if (slot == 3u) {
        gs = g3;
    }
    let r = h2.z * ih;
    G = G + gs * (cd.x * (tl + r * ds));
    H = H + outer(gs, gs) * (cd.x * (1.0 + r));
}

fn link_contrib(base: u32, slot: u32) {
    let l0 = elemu[base];
    let l1 = f4(elemu[base + 1u]);
    let l2 = f4(elemu[base + 2u]);
    let n = ub(l0.w);
    var dd = vs[ub(l0.x)].x.xyz * l1.x + vs[ub(l0.y)].x.xyz * l1.y;
    if (n > 2u) {
        dd = dd + vs[ub(l0.z)].x.xyz * l1.z;
    }
    let k = l1.w * su.p1.z;
    let cs = l1[slot];
    let len = length(dd);
    if (len < 1.0e-9) {
        H = H + ident() * (k * cs * cs);
        return;
    }
    let nn = dd / len;
    let c = len - l2.x;
    let nm = outer(nn, nn);
    var tf = 0.0;
    if (c > 0.0) {
        tf = k * c / len;
    }
    let m = nm * k + (ident() - nm) * tf;
    G = G + nn * (k * c * cs);
    H = H + m * (cs * cs);
}

struct Ct {
    ok: bool,
    n: vec3<f32>,
    depth: f32,
}

struct Cp {
    p: vec3<f32>,
    feat: u32,
}

fn closest_tri(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> Cp {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if (d1 <= 0.0 && d2 <= 0.0) {
        return Cp(a, 1u);
    }
    let bp = p - b;
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if (d3 >= 0.0 && d4 <= d3) {
        return Cp(b, 2u);
    }
    let vc = d1 * d4 - d3 * d2;
    if (vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0) {
        let v = d1 / (d1 - d3);
        return Cp(a + ab * v, 4u);
    }
    let cp = p - c;
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if (d6 >= 0.0 && d5 <= d6) {
        return Cp(c, 3u);
    }
    let vb = d5 * d2 - d1 * d6;
    if (vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0) {
        let w = d2 / (d2 - d6);
        return Cp(a + ac * w, 6u);
    }
    let va = d3 * d6 - d5 * d4;
    if (va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0) {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return Cp(b + (c - b) * w, 5u);
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    return Cp(a + ab * v + ac * w, 0u);
}

fn ray_tri(o: vec3<f32>, d: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> f32 {
    let e1 = b - a;
    let e2 = c - a;
    let p = cross(d, e2);
    let det = dot(e1, p);
    if (abs(det) < 1.0e-12) {
        return -1.0;
    }
    let inv = 1.0 / det;
    let s = o - a;
    let u = dot(s, p) * inv;
    if (u < 0.0 || u > 1.0) {
        return -1.0;
    }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if (v < 0.0 || u + v > 1.0) {
        return -1.0;
    }
    let t = dot(e2, q) * inv;
    if (t < 0.0 || t > 1.0) {
        return -1.0;
    }
    return t;
}

struct MeshInfo {
    origin: vec3<f32>,
    cell: f32,
    dims: vec3<u32>,
    tdata: u32,
    cstart: u32,
}

fn mesh_info(hb: u32) -> MeshInfo {
    let c0 = colu[hb];
    let c1 = cf(hb + 1u);
    let c2 = colu[hb + 2u];
    return MeshInfo(c1.xyz, c1.w, vec3<u32>(ub(c2.x), ub(c2.y), ub(c2.z)), ub(c0.y), ub(c0.z));
}

fn cell_range(mi: MeshInfo, lo: vec3<f32>, hi: vec3<f32>) -> array<vec3<i32>, 2> {
    let dm = vec3<i32>(mi.dims) - vec3<i32>(1);
    let a = vec3<i32>(floor((lo - mi.origin) / mi.cell));
    let b = vec3<i32>(floor((hi - mi.origin) / mi.cell));
    return array<vec3<i32>, 2>(clamp(a, vec3<i32>(0), dm), clamp(b, vec3<i32>(0), dm));
}

fn outside(mi: MeshInfo, lo: vec3<f32>, hi: vec3<f32>) -> bool {
    let ext = vec3<f32>(mi.dims) * mi.cell + mi.origin;
    return any(hi < mi.origin) || any(lo > ext);
}

fn mesh_signed(hb: u32, x: vec3<f32>, range: f32) -> vec4<f32> {
    let mi = mesh_info(hb);
    let lo = x - vec3<f32>(range);
    let hi = x + vec3<f32>(range);
    if (outside(mi, lo, hi)) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let cr = cell_range(mi, lo, hi);
    var best2 = range * range;
    var found = false;
    var bt = 0u;
    var bp = vec3<f32>(0.0);
    var bf = 0u;
    for (var k = cr[0].z; k <= cr[1].z; k++) {
        for (var j = cr[0].y; j <= cr[1].y; j++) {
            for (var i = cr[0].x; i <= cr[1].x; i++) {
                let c = (u32(k) * mi.dims.y + u32(j)) * mi.dims.x + u32(i);
                let s = cidx[mi.cstart + c];
                let e = cidx[mi.cstart + c + 1u];
                for (var q = s; q < e; q++) {
                    let t = cidx[q];
                    let tb = mi.tdata + t * 10u;
                    let r = closest_tri(x, cf(tb).xyz, cf(tb + 1u).xyz, cf(tb + 2u).xyz);
                    let dv = r.p - x;
                    let d2 = dot(dv, dv);
                    if (d2 <= best2) {
                        best2 = d2;
                        found = true;
                        bt = t;
                        bp = r.p;
                        bf = r.feat;
                    }
                }
            }
        }
    }
    if (!found) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let tb = mi.tdata + bt * 10u;
    var n = cf(tb + 3u).xyz;
    if (bf >= 1u) {
        n = cf(tb + 3u + bf).xyz;
    }
    let d = sqrt(best2);
    let dir = x - bp;
    var side = 1.0;
    if (dot(dir, n) < 0.0) {
        side = -1.0;
    }
    if (d < 1.0e-9) {
        side = 1.0;
    }
    var normal = n;
    if (d >= 1.0e-6) {
        normal = (dir / d) * side;
    }
    if (dot(normal, normal) < 0.5) {
        normal = n;
    }
    if (side < 0.0) {
        normal = n;
    }
    return vec4<f32>(d * side, normal);
}

fn mesh_sweep(hb: u32, a0: vec3<f32>, a1: vec3<f32>, offset: f32) -> vec4<f32> {
    let mi = mesh_info(hb);
    let lo = min(a0, a1);
    let hi = max(a0, a1);
    if (outside(mi, lo, hi)) {
        return vec4<f32>(0.0);
    }
    let cr = cell_range(mi, lo, hi);
    let d = a1 - a0;
    var tmax = 1.0;
    var found = false;
    var bt = 0u;
    for (var k = cr[0].z; k <= cr[1].z; k++) {
        for (var j = cr[0].y; j <= cr[1].y; j++) {
            for (var i = cr[0].x; i <= cr[1].x; i++) {
                let c = (u32(k) * mi.dims.y + u32(j)) * mi.dims.x + u32(i);
                let s = cidx[mi.cstart + c];
                let e = cidx[mi.cstart + c + 1u];
                for (var q = s; q < e; q++) {
                    let t = cidx[q];
                    let tb = mi.tdata + t * 10u;
                    let h = ray_tri(a0, d, cf(tb).xyz, cf(tb + 1u).xyz, cf(tb + 2u).xyz);
                    if (h >= 0.0 && h <= tmax) {
                        tmax = h;
                        found = true;
                        bt = t;
                    }
                }
            }
        }
    }
    if (!found) {
        return vec4<f32>(0.0);
    }
    var n = cf(mi.tdata + bt * 10u + 3u).xyz;
    if (dot(d, n) > 0.0) {
        n = -n;
    }
    let hit = a0 + d * tmax;
    let along = dot(a1 - hit, n);
    return vec4<f32>(n, max(offset - along, 0.0));
}

fn signed_dist(ci: u32, x: vec3<f32>, range: f32) -> vec4<f32> {
    let hb = ci * 3u;
    let kind = colu[hb].x;
    let c1 = cf(hb + 1u);
    let c2 = cf(hb + 2u);
    if (kind == 0u) {
        return vec4<f32>(dot(x - c1.xyz, c2.xyz), c2.xyz);
    }
    if (kind == 1u) {
        let d = x - c1.xyz;
        let l = length(d);
        var n = vec3<f32>(0.0, 1.0, 0.0);
        if (l > 1.0e-9) {
            n = d / l;
        }
        return vec4<f32>(l - c1.w, n);
    }
    if (kind == 2u) {
        let ab = c2.xyz - c1.xyz;
        let len2 = dot(ab, ab);
        var t = 0.0;
        if (len2 > 1.0e-12) {
            t = clamp(dot(x - c1.xyz, ab) / len2, 0.0, 1.0);
        }
        let d = x - (c1.xyz + ab * t);
        let l = length(d);
        var n = vec3<f32>(0.0, 1.0, 0.0);
        if (l > 1.0e-9) {
            n = d / l;
        }
        return vec4<f32>(l - c1.w, n);
    }
    return mesh_signed(hb, x, range);
}

fn is_mesh(ci: u32) -> bool {
    return colu[ci * 3u].x == 3u;
}

fn contact_at(ci: u32, x: vec3<f32>, offset: f32) -> Ct {
    return contact_range(ci, x, offset, max(offset, 1.0e-6));
}

fn contact_range(ci: u32, x: vec3<f32>, offset: f32, range: f32) -> Ct {
    let hb = ci * 3u;
    let kind = colu[hb].x;
    let sd = signed_dist(ci, x, range);
    if (kind == 3u && sd.x == 0.0 && sd.y == 0.0 && sd.z == 0.0 && sd.w == 0.0) {
        return Ct(false, vec3<f32>(0.0), 0.0);
    }
    if (sd.x < offset) {
        return Ct(true, sd.yzw, offset - sd.x);
    }
    return Ct(false, vec3<f32>(0.0), 0.0);
}

fn deepest(x: vec3<f32>, offset: f32) -> Ct {
    var best = Ct(false, vec3<f32>(0.0), 0.0);
    for (var c = 0u; c < su.u0.y; c++) {
        let ct = contact_at(c, x, offset);
        if (ct.ok && ct.depth > 0.0 && (!best.ok || best.depth < ct.depth)) {
            best = ct;
        }
    }
    return best;
}

fn newton(hm: mat3x3<f32>, g: vec3<f32>) -> vec3<f32> {
    let sc = 1.0 / max(max(hm[0][0], hm[1][1]), max(hm[2][2], 1.0e-20));
    let a = hm[0][0] * sc;
    let b = hm[1][0] * sc;
    let c = hm[2][0] * sc;
    let d = hm[1][1] * sc;
    let e = hm[2][1] * sc;
    let f = hm[2][2] * sc;
    let c0 = d * f - e * e;
    let c1 = c * e - b * f;
    let c2 = b * e - c * d;
    let det = a * c0 + b * c1 + c * c2;
    if (abs(det) < 1.0e-20 || det != det) {
        return vec3<f32>(0.0);
    }
    let c3 = a * f - c * c;
    let c4 = b * c - a * e;
    let c5 = a * d - b * b;
    let r = -g * sc;
    let inv = 1.0 / det;
    return vec3<f32>(c0 * r.x + c1 * r.y + c2 * r.z, c1 * r.x + c3 * r.y + c4 * r.z, c2 * r.x + c4 * r.y + c5 * r.z) * inv;
}

fn finite3(v: vec3<f32>) -> bool {
    return all(abs(v) < vec3<f32>(1.0e30));
}

@compute @workgroup_size(64)
fn begin_a(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = su.u0.x;
    if (i >= n) {
        return;
    }
    vs[i].xn = vs[i].x;
    vs[i].v = vs[i].v * su.p1.w;
    var ext = vec3<f32>(0.0);
    let hdr = su.u2.y + 4u * i;
    let st = idx[hdr];
    let nt = idx[hdr + 1u];
    for (var j = 0u; j < nt; j++) {
        let en = idx[su.u2.z + st + j];
        let base = su.u1.x + (en >> 2u) * 4u;
        let kk = f4(elemu[base + 3u]);
        if (kk.w != 0.0) {
            let t0 = elemu[base];
            let p0 = vs[ub(t0.x)].x.xyz;
            let p1 = vs[ub(t0.y)].x.xyz;
            let p2 = vs[ub(t0.z)].x.xyz;
            ext = ext + cross(p1 - p0, p2 - p0) * (kk.w / 6.0);
        }
    }
    vs[i].e = vec4<f32>(ext + vinfo[i].ext.xyz, 0.0);
}

@compute @workgroup_size(64)
fn begin_b(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = su.u0.x;
    if (i >= n) {
        return;
    }
    let info = vinfo[i];
    let h = su.gh.w;
    let frac = su.p0.x;
    var s = vs[i];
    if (info.m.z > 0.5) {
        let pos = info.pa.xyz + (info.pb.xyz - info.pa.xyz) * frac;
        s.x = vec4<f32>(pos, 0.0);
        s.y = vec4<f32>(pos, 0.0);
        s.e = vec4<f32>(s.e.xyz, 0.0);
        vs[i] = s;
        return;
    }
    let g = su.gh.xyz;
    let gl = length(g);
    var gdir = vec3<f32>(0.0);
    if (gl > 0.0) {
        gdir = g / gl;
    }
    let vp = s.v.xyz * su.p0.y;
    let a = g + s.e.xyz / info.m.x;
    s.y = vec4<f32>(s.xn.xyz + vp * h + a * (h * h), 0.0);
    let along = clamp(dot(s.a.xyz, gdir), 0.0, gl);
    s.x = vec4<f32>(s.xn.xyz + vp * h + gdir * along * (h * h), 0.0);
    s.e = vec4<f32>(s.e.xyz, 0.0);
    vs[i] = s;
}

@compute @workgroup_size(64)
fn hinge_eval(@builtin(global_invocation_id) gid: vec3<u32>) {
    let k = gid.x;
    if (k >= su.u0.w) {
        return;
    }
    hinge_eval_store(su.u1.y + k * 3u, k);
}

@compute @workgroup_size(64)
fn solve(@builtin(global_invocation_id) gid: vec3<u32>) {
    let k = gid.x;
    if (k >= cu.r.y) {
        return;
    }
    let i = idx[su.u2.x + cu.r.x + k];
    let info = vinfo[i];
    if (info.m.z > 0.5) {
        return;
    }
    let h = su.gh.w;
    let ih = 1.0 / h;
    let ih2 = ih * ih;
    let m = info.m.x;
    let xi = vs[i].x.xyz;
    let xn = vs[i].xn.xyz;
    G = (xi - vs[i].y.xyz) * (m * ih2);
    H = ident() * (m * ih2);
    let hdr = su.u2.y + 4u * i;
    let st = idx[hdr];
    let nt = idx[hdr + 1u];
    let nh = idx[hdr + 2u];
    let nl = idx[hdr + 3u];
    var p = su.u2.z + st;
    for (var j = 0u; j < nt; j++) {
        let en = idx[p + j];
        tri_contrib(su.u1.x + (en >> 2u) * 4u, en & 3u, ih);
    }
    p = p + nt;
    for (var j = 0u; j < nh; j++) {
        let en = idx[p + j];
        hinge_contrib(su.u1.y + (en >> 2u) * 3u, en & 3u, ih, en >> 2u);
    }
    p = p + nh;
    for (var j = 0u; j < nl; j++) {
        let en = idx[p + j];
        link_contrib(su.u1.z + (en >> 2u) * 3u, en & 3u);
    }
    if (su.u0.z == i) {
        let kg = m * ih2 * GRAB_K;
        G = G + (xi - su.tg.xyz) * kg;
        H = H + ident() * kg;
    }
    if (su.u0.y > 0u) {
        let offset = su.p0.z;
        let ct = deepest(xi, offset);
        if (ct.ok) {
            var mu = su.p0.w;
            if (info.m.y > 0.0) {
                mu = sqrt(su.p0.w * info.m.y);
            }
            let kc = CONTACT_K * m * ih2;
            let nn = outer(ct.n, ct.n);
            G = G - ct.n * (kc * ct.depth);
            H = H + nn * kc;
            if (mu > 0.0) {
                let disp = xi - xn;
                let lam = kc * ct.depth;
                let ut = disp - ct.n * dot(disp, ct.n);
                let s = sqrt(dot(ut, ut) + FRICTION_EPS * FRICTION_EPS);
                let w = min(mu * lam / s, kc);
                G = G + ut * w;
                H = H + (ident() - nn) * w;
            }
        }
    }
    let dx = newton(H, G);
    let np = xi + dx;
    if (finite3(np)) {
        vs[i].x = vec4<f32>(np, 0.0);
    }
}

@compute @workgroup_size(64)
fn finish(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = su.u0.x;
    if (i >= n) {
        return;
    }
    let info = vinfo[i];
    let h = su.gh.w;
    let offset = su.p0.z;
    var s = vs[i];
    var x = s.x.xyz;
    var flag = 0.0;
    var pre = x;
    var bn = vec3<f32>(0.0);
    var bestd = 0.0;
    if (info.m.z < 0.5 && su.u0.y > 0u) {
        let cap = APPROACH_CAP * offset;
        if (cap > 0.0 && dot(x - s.xn.xyz, x - s.xn.xyz) > cap * cap) {
            let zone = max(APPROACH_ZONE * offset, 20.0);
            var nsd = 1.0e30;
            var nn = vec3<f32>(0.0);
            for (var c = 0u; c < su.u0.y; c++) {
                let sd = signed_dist(c, s.xn.xyz, zone);
                if (dot(sd.yzw, sd.yzw) > 0.5 && sd.x < nsd) {
                    nsd = sd.x;
                    nn = sd.yzw;
                }
            }
            let inward = dot(s.xn.xyz - x, nn);
            if (nsd < zone && inward > cap) {
                x = x + nn * (inward - cap);
            }
        }
        pre = x;
        let moved = length(x - s.xn.xyz);
        if (moved > 0.5 * max(offset, 0.5)) {
            for (var c = 0u; c < su.u0.y; c++) {
                if (is_mesh(c)) {
                    let sw = mesh_sweep(c * 3u, s.xn.xyz, x, offset);
                    if (sw.w > 0.0) {
                        x = x + sw.xyz * sw.w;
                    }
                }
            }
        }
        for (var rp = 0u; rp < 2u; rp++) {
            var any_moved = false;
            for (var c = 0u; c < su.u0.y; c++) {
                let ct = contact_range(c, x, offset, max(offset, 1.0e-6) + DEEP_REACH);
                if (ct.ok && ct.depth > 1.0e-5) {
                    x = x + ct.n * ct.depth;
                    any_moved = true;
                    if (ct.depth > bestd) {
                        bestd = ct.depth;
                        bn = ct.n;
                    }
                    flag = 1.0;
                }
            }
            if (!any_moved) {
                break;
            }
        }
    }
    var nv = (x - s.xn.xyz) / h;
    if (flag > 0.5) {
        nv = (pre - s.xn.xyz) / h;
        let vn = dot(nv, bn);
        if (vn < 0.0) {
            nv = nv - bn * vn;
        }
    }
    s.a =vec4<f32>((nv - s.v.xyz) / h, 0.0);
    s.v = vec4<f32>(nv, 0.0);
    s.x = vec4<f32>(x, 0.0);
    vs[i] = s;
    outb[2u * i] = vec4<f32>(x, length(nv));
    outb[2u * i + 1u] = vec4<f32>(flag, 0.0, 0.0, 0.0);
}
