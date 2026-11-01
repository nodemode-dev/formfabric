mod support;

use ffb_fabric::Physical;
use ffb_sim::{compare_positions, reference_step, Cloth, Material, Method, Params, PinPoint, Solver};
use support::{arg, name};

const MARKS: [usize; 5] = [10, 20, 30, 60, 90];

fn build(spacing: f32) -> Cloth {
    let n = (1000.0 / spacing).round() as usize + 1;
    let mut cloth = Cloth { materials: vec![Material::from_physical(&Physical::default())], ..Cloth::default() };
    support::add_grid(&mut cloth, n, n, spacing, 0, &|s, t| [s - 500.0, -t, 0.0]);
    for v in [0, n - 1] {
        cloth.pins.push(PinPoint { vertex: v as u32, target: [v as f32 * spacing - 500.0, 0.0, 0.0] });
    }
    cloth
}

struct Run {
    snaps: Vec<Solver>,
    ms: f32,
    its: f32,
    end: Solver,
}

fn run(cloth: &Cloth, params: Params, frames: usize) -> Run {
    let mut s = Solver::new(cloth.clone(), params);
    let mut total = 0.0;
    let mut its = 0u64;
    let mut snaps = Vec::new();
    for f in 0..frames {
        let st = s.step();
        total += st.ms;
        its += st.iterations as u64;
        if MARKS.contains(&(f + 1)) {
            snaps.push(s.clone());
        }
    }
    Run { snaps, ms: total / frames as f32, its: its as f32 / frames as f32, end: s }
}

fn row(label: &str, a: &Run, b: &Run) {
    let mut line = format!("  {label:14}");
    for (x, y) in a.snaps.iter().zip(&b.snaps) {
        let (mean, _) = compare_positions(x, y);
        line.push_str(&format!(" {:8.3}", mean));
    }
    println!("{line}");
}

fn main() {
    let spacing = arg("spacing", 20.0);
    let frames = arg("frames", 90.0) as usize;
    let cloth = build(spacing);
    println!("vertices {} frames {}  columns: mean distance (mm) at frames {:?} (1 frame = 1/30 s)", cloth.positions.len(), frames, MARKS);
    let mut reference: Option<Run> = None;
    if arg("ref", 1.0) > 0.0 {
        let p = Params { substeps: 5, ..Params::default() };
        let mut cg = Solver::new(cloth.clone(), p);
        let mut snaps = Vec::new();
        let mut total = 0.0;
        for f in 0..frames {
            total += reference_step(&mut cg).ms;
            if MARKS.contains(&(f + 1)) {
                snaps.push(cg.clone());
            }
        }
        println!("reference (newton 6, cg 1e-6, 5 substeps): {:.1} ms/frame, KE {:.4}, strain +{:.4}", total / frames as f32, cg.kinetic_energy(), cg.max_stretch());
        reference = Some(Run { snaps, ms: total / frames as f32, its: 0.0, end: cg });
    }
    for m in support::methods() {
        let params = support::params_for(m);
        let r = run(&cloth, params, frames);
        let p = r.end.implicit_profile();
        println!(
            "{} substeps {} iterations {} newton {} cg {} tol {}: {:.2} ms/frame, its/frame {:.1}, KE {:.4}, strain +{:.4} -{:.4}, assemble {:.2} pcg {:.2}",
            name(m),
            params.substeps,
            params.iterations,
            params.newton,
            params.cg_iterations,
            params.cg_tolerance,
            r.ms,
            r.its,
            r.end.kinetic_energy(),
            r.end.max_stretch(),
            r.end.max_compression(),
            p[0] / frames as f64,
            p[1] / frames as f64
        );
        if let Some(refr) = &reference {
            row("vs reference", &r, refr);
        }
        let fine = Params { substeps: params.substeps * 4, ..params };
        let f = run(&cloth, fine, frames);
        row("vs 4x time", &r, &f);
        println!("  4x time resolution: {:.2} ms/frame", f.ms);
        if m == Method::Implicit {
            if let Some(refr) = &reference {
                row("4x vs reference", &f, refr);
            }
        }
    }
}
