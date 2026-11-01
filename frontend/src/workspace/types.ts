export type Vec = [number, number];
export type PointKind = 'corner' | 'curve' | 'smooth';

export interface Pt { p: Vec; kind: PointKind; h_in: Vec | null; h_out: Vec | null }

export interface Inner { id: number; kind: string; points: Pt[]; closed: boolean; fold_angle: number; fold_strength: number }

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
}

export interface Garment { units: string; panels: Panel[]; selection?: unknown }

export interface PanelMeasure {
  panel: number;
  name: string;
  area: number;
  perimeter: number;
  segments: Array<{ index: number; length: number; chord: number; curved: boolean }>;
}
