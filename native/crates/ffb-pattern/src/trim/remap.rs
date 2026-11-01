use crate::sew::{self, end_valid, Op};
use crate::{EdgeKind, Garment, Seam, SeamEnd};

fn carrier(a: &[SeamEnd], b: &[SeamEnd]) -> Seam {
    Seam { a: a.to_vec(), b: b.to_vec(), ..Seam::default() }
}

fn remap(garment: &mut Garment, ops: &[Op]) {
    let real = std::mem::take(&mut garment.seams);
    let mut fake: Vec<Seam> = Vec::new();
    fake.extend(garment.stitches.iter().map(|s| carrier(&s.spans, &s.spans)));
    fake.extend(garment.zippers.iter().map(|z| carrier(&z.a, &z.b)));
    fake.extend(garment.finishes.iter().map(|f| carrier(&f.spans, &f.spans)));
    garment.seams = fake;
    sew::apply_pending(garment, ops.to_vec());
    let mut moved = std::mem::replace(&mut garment.seams, real).into_iter();
    for s in garment.stitches.iter_mut() {
        s.spans = moved.next().map(|x| x.a).unwrap_or_default();
    }
    for z in garment.zippers.iter_mut() {
        if let Some(x) = moved.next() {
            z.a = x.a;
            z.b = x.b;
        }
    }
    for f in garment.finishes.iter_mut() {
        f.spans = moved.next().map(|x| x.a).unwrap_or_default();
    }
}

fn tidy(garment: &Garment, ends: &mut Vec<SeamEnd>) {
    for e in ends.iter_mut() {
        e.t0 = e.t0.clamp(0.0, 1.0);
        e.t1 = e.t1.clamp(0.0, 1.0);
    }
    ends.retain(|e| end_valid(garment, e));
}

fn needs_strip(kind: EdgeKind) -> bool {
    matches!(kind, EdgeKind::Piping | EdgeKind::Binding | EdgeKind::Elastic)
}

pub fn prune(garment: &mut Garment) {
    let mut stitches = std::mem::take(&mut garment.stitches);
    for s in stitches.iter_mut() {
        tidy(garment, &mut s.spans);
    }
    stitches.retain(|s| !s.spans.is_empty());
    garment.stitches = stitches;
    let mut zippers = std::mem::take(&mut garment.zippers);
    for z in zippers.iter_mut() {
        tidy(garment, &mut z.a);
        tidy(garment, &mut z.b);
    }
    zippers.retain(|z| !z.a.is_empty() && !z.b.is_empty());
    garment.zippers = zippers;
    let panels: Vec<u32> = garment.panels.iter().map(|p| p.id).collect();
    garment.hardware.retain(|h| panels.contains(&h.anchor.panel));
    let live: Vec<u32> = garment.hardware.iter().map(|h| h.id).collect();
    for h in garment.hardware.iter_mut() {
        if h.fastened_to.is_some_and(|x| !live.contains(&x)) {
            h.fastened_to = None;
        }
    }
    let mut finishes = std::mem::take(&mut garment.finishes);
    for f in finishes.iter_mut() {
        tidy(garment, &mut f.spans);
        f.generated.retain(|g| panels.contains(g));
    }
    let (keep, dead): (Vec<_>, Vec<_>) = finishes.into_iter().partition(|f| !f.spans.is_empty() && (!needs_strip(f.kind) || !f.generated.is_empty()));
    garment.finishes = keep;
    let doomed: Vec<u32> = dead.iter().flat_map(|f| f.generated.iter().copied()).collect();
    if !doomed.is_empty() {
        garment.panels.retain(|p| !doomed.contains(&p.id));
        sew::prune(garment);
    }
    let seams: Vec<u32> = garment.seams.iter().map(|s| s.id).collect();
    garment.ripped.retain(|r| seams.contains(r));
}

pub fn follow(garment: &mut Garment, ops: &[Op]) {
    if !ops.is_empty() && (!garment.stitches.is_empty() || !garment.zippers.is_empty() || !garment.finishes.is_empty()) {
        remap(garment, ops);
    }
    prune(garment);
}
