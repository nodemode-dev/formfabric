import { geoOf, type PanelGeo } from '../../../src/workspace/geom';
import type { Inner, Panel, Pt, Vec } from '../../../src/workspace/types';

export const corner = (x: number, y: number): Pt => ({ p: [x, y], kind: 'corner', h_in: null, h_out: null });

export function panelOf(id: number, outline: Pt[], over: Partial<Panel> = {}): Panel {
  return { id, name: `P${id}`, outline, inner: [], position: [0, 0], rotation: 0, mirror: false, grain: 0, particle_distance: 10, layer: 0, ...over };
}

export function square(id: number, at: Vec, size: number, over: Partial<Panel> = {}): Panel {
  return panelOf(id, [corner(0, 0), corner(size, 0), corner(size, size), corner(0, size)], { position: at, ...over });
}

export function openInner(id: number, a: Vec, b: Vec): Inner {
  return { id, kind: 'line', points: [corner(a[0], a[1]), corner(b[0], b[1])], closed: false, fold_angle: 0, fold_strength: 0 };
}

export const KAPPA = (4 / 3) * (Math.SQRT2 - 1);

export function quarterDisc(id: number, r: number): Panel {
  const a: Pt = { p: [r, 0], kind: 'curve', h_in: null, h_out: [0, KAPPA * r] };
  const b: Pt = { p: [0, r], kind: 'curve', h_in: [KAPPA * r, 0], h_out: null };
  return panelOf(id, [a, b, corner(0, 0)]);
}

export function geoMap(...panels: Panel[]): Map<number, PanelGeo> {
  return new Map(panels.map((p) => [p.id, geoOf(p)]));
}
