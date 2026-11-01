export interface BodyInfo {
  id: number;
  name: string;
  object: number;
  source: string | null;
  license: string | null;
  vertices: number;
  triangles: number;
  joints: number;
  visible: boolean;
  skin_offset: number;
  friction: number;
  posed_joints: string[];
  measurements: Record<string, number>;
}

export interface LibraryEntry {
  name: string;
  license: string | null;
  built_in: boolean;
  path?: string;
}

export interface BodyList {
  bodies: BodyInfo[];
  library: LibraryEntry[];
  poses: string[];
}

export interface JointInfo {
  index: number;
  name: string;
  pose: [number, number, number, number];
}

export interface JointsReply {
  id: number;
  count: number;
  joints: JointInfo[];
}

export interface MeasureReply {
  id: number;
  unit: string;
  measurements: Record<string, number>;
}

export interface PlaceItem {
  key: string;
  name: string;
  offset?: number;
}

export interface Proposal {
  key: string;
  name: string;
  mismatch: number;
  args: Record<string, unknown>;
}

export const MEASURES: Array<[string, string]> = [
  ['height', 'Height'],
  ['chest', 'Chest'],
  ['waist', 'Waist'],
  ['hip', 'Hip'],
  ['inseam', 'Inseam'],
  ['shoulder_width', 'Shoulder width'],
  ['arm_length', 'Arm length'],
];

export const MAIN_JOINTS = [
  'spine', 'spine01', 'spine03', 'spine05', 'chest', 'neck', 'neck01', 'head',
  'pelvis', 'l_clavicle', 'r_clavicle', 'clavicle.L', 'clavicle.R',
  'l_shoulder', 'r_shoulder', 'upperarm01.L', 'upperarm01.R',
  'l_elbow', 'r_elbow', 'lowerarm01.L', 'lowerarm01.R',
  'l_wrist', 'r_wrist', 'wrist.L', 'wrist.R',
  'l_hip', 'r_hip', 'upperleg01.L', 'upperleg01.R',
  'l_knee', 'r_knee', 'lowerleg01.L', 'lowerleg01.R',
  'l_ankle', 'r_ankle', 'foot.L', 'foot.R',
];

export const IMPORT_EXT = ['obj', 'glb', 'gltf', 'fbx'];
