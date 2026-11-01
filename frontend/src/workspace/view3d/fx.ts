export const MAX_LIGHTS = 4;

export interface MapFx {
  on: boolean;
  ramp: WebGLTexture | null;
  min: number;
  max: number;
}

export interface StageFx {
  on: boolean;
  count: number;
  pos: Float32Array;
  dir: Float32Array;
  col: Float32Array;
  shadowIdx: number;
  shadowMat: Float32Array;
  shadowTex: WebGLTexture | null;
  shadowTexel: number;
  sh: Float32Array;
  hasSH: boolean;
  ambient: Float32Array;
  exposure: number;
  rotation: number;
}

export interface Fx {
  map: MapFx;
  stage: StageFx;
}

export function newFx(): Fx {
  return {
    map: { on: false, ramp: null, min: 0, max: 1 },
    stage: {
      on: false,
      count: 0,
      pos: new Float32Array(MAX_LIGHTS * 4),
      dir: new Float32Array(MAX_LIGHTS * 3),
      col: new Float32Array(MAX_LIGHTS * 3),
      shadowIdx: -1,
      shadowMat: new Float32Array(16),
      shadowTex: null,
      shadowTexel: 1 / 2048,
      sh: new Float32Array(27),
      hasSH: false,
      ambient: new Float32Array([0.12, 0.12, 0.14]),
      exposure: 1,
      rotation: 0,
    },
  };
}

export const SHADOW_GLSL = `
uniform sampler2DShadow uShadow;
uniform mat4 uShadowM;
uniform float uShadowTexel;
float shadowAt(vec3 w, vec3 n) {
  vec4 s = uShadowM * vec4(w + n * 4.0, 1.0);
  vec3 p = s.xyz / s.w * 0.5 + 0.5;
  if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0 || p.z > 1.0) return 1.0;
  float sum = 0.0;
  for (int x = -1; x <= 1; x++) {
    for (int y = -1; y <= 1; y++) {
      sum += texture(uShadow, vec3(p.xy + vec2(float(x), float(y)) * uShadowTexel, p.z - 0.0006));
    }
  }
  return sum / 9.0;
}`;
