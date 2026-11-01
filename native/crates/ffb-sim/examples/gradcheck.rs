use ffb_sim::energy::{HingeElem, LinkElem, NonLinear, TriElem};
use glam::DVec3;

fn rnd(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*seed >> 33) as f64) / (1u64 << 31) as f64 - 0.5
}

fn check<const N: usize>(name: &str, p: [DVec3; N], energy: &dyn Fn([DVec3; N]) -> f64, grad: &dyn Fn([DVec3; N], usize) -> DVec3) {
    let mut worst: f64 = 0.0;
    let mut scale: f64 = 1.0e-12;
    for s in 0..N {
        let g = grad(p, s);
        scale = scale.max(g.length());
        for axis in 0..3 {
            let eps = 1.0e-6;
            let mut a = p;
            let mut b = p;
            a[s][axis] += eps;
            b[s][axis] -= eps;
            let fd = (energy(a) - energy(b)) / (2.0 * eps);
            worst = worst.max((fd - g[axis]).abs());
        }
    }
    println!("{name}: max abs error {worst:.3e} against gradient scale {scale:.3e} (relative {:.2e})", worst / scale);
}

fn main() {
    let mut seed = 7u64;
    let nl = NonLinear { on: true, gain: 40.0 };
    let tri = TriElem { v: [0, 1, 2], alpha: [-0.1, 0.05, 0.05], beta: [-0.05, 0.1, -0.05], area: 200.0, ku: 5.0e6, kv: 6.0e6, ks: 5.0e5, kd: 0.0 };
    let p: [DVec3; 3] = std::array::from_fn(|i| DVec3::new(i as f64 * 20.0, (i * i) as f64 * 5.0, 0.0) + DVec3::new(rnd(&mut seed), rnd(&mut seed), rnd(&mut seed)) * 8.0);
    check("membrane", p, &|q| tri.energy(q, nl), &|q, s| tri.eval(q, nl).grad(s));

    let hinge = HingeElem { v: [0, 1, 2, 3], coef: 3000.0, theta0: 0.4, h_rest: [10.0, 10.0], buckle_ratio: 0.0, buckle_floor: 1.0, kd: 0.0 };
    let q: [DVec3; 4] = [DVec3::new(0.0, 0.0, 0.0), DVec3::new(20.0, 0.0, 0.0), DVec3::new(8.0, 14.0, 3.0), DVec3::new(11.0, -13.0, 5.0)];
    check("hinge", q, &|r| hinge.eval(r).energy(), &|r, s| hinge.eval(r).grad(s));

    let link = LinkElem { v: [0, 1, 2], c: [1.0, -0.7, -0.3], n: 3, k: 1.0e4, rest: 2.0 };
    let r: [DVec3; 3] = [DVec3::new(0.0, 0.0, 0.0), DVec3::new(20.0, 1.0, 0.0), DVec3::new(5.0, 9.0, 3.0)];
    check("link", r, &|a| link.energy(a), &|a, s| link.eval(a).grad(s));

    let d: [DVec3; 3] = std::array::from_fn(|_| DVec3::new(rnd(&mut seed), rnd(&mut seed), rnd(&mut seed)));
    let rate = 0.3;
    let asm = tri.assemble(p, d, rate, nl, true);
    let ev = tri.eval(p, nl);
    let du = d[0] * tri.alpha[0] + d[1] * tri.alpha[1] + d[2] * tri.alpha[2];
    let dv = d[0] * tri.beta[0] + d[1] * tri.beta[1] + d[2] * tri.beta[2];
    let mut worst_g: f64 = 0.0;
    let mut worst_h: f64 = 0.0;
    for (k, &(s, t)) in ffb_sim::energy::TRI_PAIRS.iter().enumerate() {
        let h = ev.hess(s, t) * (1.0 + rate);
        let c = h.to_cols_array();
        for r in 0..3 {
            for q in 0..3 {
                worst_h = worst_h.max((asm.blk[k][3 * r + q] - c[3 * q + r]).abs() / (c[3 * q + r].abs() + 1.0));
            }
        }
    }
    for s in 0..3 {
        let g = ev.grad(s) + ev.damp_d(s, du, dv) * rate;
        worst_g = worst_g.max((asm.grad[s] - g).length() / (g.length() + 1.0));
    }
    println!("membrane assemble against eval: gradient relative {worst_g:.2e}, hessian relative {worst_h:.2e}");
}
