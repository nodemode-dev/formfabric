pub type V2 = [f64; 2];
pub type Bounds = [V2; 2];

pub const EPS: f64 = 1e-9;

pub fn add(a: V2, b: V2) -> V2 {
    [a[0] + b[0], a[1] + b[1]]
}

pub fn sub(a: V2, b: V2) -> V2 {
    [a[0] - b[0], a[1] - b[1]]
}

pub fn mul(a: V2, k: f64) -> V2 {
    [a[0] * k, a[1] * k]
}

pub fn dot(a: V2, b: V2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

pub fn cross(a: V2, b: V2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

pub fn len(a: V2) -> f64 {
    a[0].hypot(a[1])
}

pub fn dist(a: V2, b: V2) -> f64 {
    len(sub(a, b))
}

pub fn lerp(a: V2, b: V2, t: f64) -> V2 {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

pub fn unit(a: V2) -> Option<V2> {
    let l = len(a);
    (l > EPS).then(|| mul(a, 1.0 / l))
}

pub fn rotate(a: V2, degrees: f64) -> V2 {
    let (s, c) = degrees.to_radians().sin_cos();
    [a[0] * c - a[1] * s, a[0] * s + a[1] * c]
}

pub fn finite(a: V2) -> bool {
    a[0].is_finite() && a[1].is_finite()
}

pub fn bounds_of(points: impl IntoIterator<Item = V2>) -> Option<Bounds> {
    let mut it = points.into_iter();
    let first = it.next()?;
    Some(it.fold([first, first], grow))
}

pub fn grow(b: Bounds, p: V2) -> Bounds {
    [[b[0][0].min(p[0]), b[0][1].min(p[1])], [b[1][0].max(p[0]), b[1][1].max(p[1])]]
}

pub fn union(a: Bounds, b: Bounds) -> Bounds {
    grow(grow(a, b[0]), b[1])
}

pub fn center(b: Bounds) -> V2 {
    lerp(b[0], b[1], 0.5)
}

pub fn dist_to_line(p: V2, a: V2, b: V2) -> f64 {
    let ab = sub(b, a);
    let l = len(ab);
    if l < EPS {
        dist(p, a)
    } else {
        (cross(ab, sub(p, a)) / l).abs()
    }
}

pub fn closest_on_segment(p: V2, a: V2, b: V2) -> (f64, V2) {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    let t = if l2 < EPS { 0.0 } else { (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0) };
    (t, lerp(a, b, t))
}
