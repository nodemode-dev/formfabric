export type Vec = [number, number];
export type PointKind = 'corner' | 'curve' | 'smooth';

export interface Pt { p: Vec; kind: PointKind; h_in: Vec | null; h_out: Vec | null }

export type InnerRole = 'shape' | 'fold' | 'dart' | 'reference';

export interface Inner { id: number; kind: string; points: Pt[]; closed: boolean; fold_angle: number; fold_strength: number; role?: InnerRole }

export type NotchKind = 'slit' | 'v' | 't' | 'u';
export type CornerStyle = 'miter' | 'square' | 'round' | 'mirror';

export interface Notch { id: number; segment: number; t: number; kind: NotchKind; depth: number; width: number; angle: number }
export interface NotchPoint { id: number; point: Vec; normal: Vec; world: Vec }
export interface SeamAllowance { width: number; corners: CornerStyle; per_segment: Array<number | null> }
export interface PanelLink { kind: 'mirror' | 'instance'; source: number; axis: unknown }
export interface GrainEffective { from: Vec; to: Vec; computed: boolean; angle: number }

export interface Panel {
  id: number;
  name: string;
  outline: Pt[];
  inner: Inner[];
  position: Vec;
  rotation: number;
  mirror: boolean;
  grain: number;
  particle_distance: number;
  layer: number;
  seam_allowance?: SeamAllowance | null;
  seam_outline?: Pt[] | null;
  notches?: Notch[];
  notch_points?: NotchPoint[];
  grain_line?: [Vec, Vec] | null;
  grain_line_effective?: GrainEffective;
  link?: PanelLink | null;
}

export interface Garment { units: string; panels: Panel[]; selection?: unknown }

export interface PanelMeasure {
  panel: number;
  name: string;
  area: number;
  perimeter: number;
  segments: Array<{ index: number; length: number; chord: number; curved: boolean }>;
}
