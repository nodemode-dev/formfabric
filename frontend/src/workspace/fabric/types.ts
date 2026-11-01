export type Rgba = [number, number, number, number];
export type Pair = [number, number];

export type MapKey = 'base_color' | 'normal' | 'roughness' | 'metalness' | 'opacity' | 'displacement';
export type Maps = Record<MapKey, string | null>;

export interface Surface {
  color: Rgba;
  roughness: number;
  metalness: number;
  sheen: number;
  opacity: number;
  maps: Maps;
  map_scale: Pair;
  map_rotation: number;
  map_offset: Pair;
}

export type SurfaceKey = 'front' | 'back' | 'side';

export interface Look {
  kind: string;
  front: Surface;
  back: Surface | null;
  side: Surface | null;
}

export type PhysicalKey =
  | 'stretch_weft' | 'stretch_warp' | 'shear'
  | 'bending_weft' | 'bending_warp' | 'bending_bias'
  | 'buckling_ratio' | 'buckling_stiffness'
  | 'density' | 'friction' | 'thickness' | 'damping';

export type Physical = Record<PhysicalKey, number>;

export interface FabricInfo {
  id: number;
  name: string;
  preset: string | null;
  physical: Physical;
  look: Look;
  panels: number[];
  is_default: boolean;
  color: Rgba;
}

export interface Colorway {
  id: number;
  name: string;
  fabric_colors: Record<string, Rgba>;
}

export interface FabricList {
  fabrics: FabricInfo[];
  colorways: Colorway[];
  default_fabric: number | null;
  active_colorway: number | null;
}

export interface FieldDef {
  key: PhysicalKey;
  label: string;
  description: string;
  unit: string;
  min: number;
  max: number;
  scale: string;
}

export interface Preset {
  id: string;
  name: string;
  category: string;
  description: string;
  notes?: string;
  provisional: boolean;
  status: string;
  sources: string[];
  physical: Physical;
  look: Look;
}

export interface PresetsReply {
  presets: Preset[];
  fields: FieldDef[];
  default: string;
}

export interface LabFormula {
  field: string;
  expression: string;
  value: number;
  unit: string;
  sources: string[];
}

export interface LabReply {
  applied: boolean;
  changed: boolean;
  computed: string[];
  kept: string[];
  physical: Physical;
  formulas: LabFormula[];
  warnings: string[];
}

export const KINDS = ['matte', 'shiny', 'satin', 'velvet', 'leather', 'metal', 'plastic', 'glass'] as const;

export const UNIT_TEXT: Record<string, string> = { 'uN*m': 'µN·m', 'g/m2': 'g/m²', ratio: '' };

export function toHex(c: Rgba): string {
  const h = (v: number): string => Math.round(Math.min(1, Math.max(0, v)) * 255).toString(16).padStart(2, '0');
  return `#${h(c[0])}${h(c[1])}${h(c[2])}`;
}

export function fromHex(hex: string, alpha: number): Rgba {
  const n = (i: number): number => parseInt(hex.slice(i, i + 2), 16) / 255;
  return [n(1), n(3), n(5), alpha];
}
