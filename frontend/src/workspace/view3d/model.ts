import type { Garment, Panel } from '../types';

export type RGB = [number, number, number];
export type Quat = [number, number, number, number];
export type V3 = [number, number, number];

export interface SurfaceJson {
  color?: unknown;
  roughness?: number;
  sheen?: number;
  opacity?: number;
  maps?: unknown;
  map_scale?: number | number[];
  map_rotation?: number;
  map_offset?: number[];
}

export interface FabricJson {
  id: number;
  name?: string;
  look?: { kind?: string; front?: SurfaceJson; back?: SurfaceJson; side?: SurfaceJson };
}

export interface ColorwayJson {
  id: number | string;
  name?: string;
  fabric_colors?: unknown;
}

export interface Place3D {
  origin: V3;
  rotation: Quat;
  bend?: number;
}

export interface PanelFull extends Panel {
  fabric?: number | null;
  place3d?: Place3D | null;
  state?: { frozen?: boolean; disabled?: boolean };
}

export interface GarmentFull extends Omit<Garment, 'panels'> {
  panels: PanelFull[];
  fabrics?: FabricJson[];
  default_fabric?: number | null;
  colorways?: ColorwayJson[];
  active_colorway?: number | string | null;
}

export type ViewStyle = 'shaded' | 'shaded-mesh' | 'mesh' | 'panels';

export const VIEW_STYLES: ViewStyle[] = ['shaded', 'shaded-mesh', 'mesh', 'panels'];

export interface MeshData {
  local: Float32Array;
  indices: Uint32Array;
}
