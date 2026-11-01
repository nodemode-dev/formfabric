import { SHADOW_GLSL, type Fx } from './fx';
import type { Frame } from './frame';
import type { ViewStyle } from './model';
import type { Scene } from './scene';

const MESH_VS = `#version 300 es
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNrm;
layout(location = 2) in vec2 aUv;
layout(location = 3) in float aScal;
uniform mat4 uVP;
uniform mat4 uModel;
out vec3 vN;
out vec3 vW;
out vec2 vUv;
out float vS;
void main() {
  vec4 w = uModel * vec4(aPos, 1.0);
  vW = w.xyz;
  vN = mat3(uModel) * aNrm;
  vUv = aUv;
  vS = aScal;
  gl_Position = uVP * w;
}`;

const MESH_FS = `#version 300 es
precision highp float;
precision highp sampler2DShadow;
in vec3 vN;
in vec3 vW;
in vec2 vUv;
in float vS;
${SHADOW_GLSL}
uniform float uMapOn;
uniform vec2 uRange;
uniform sampler2D uRamp;
uniform float uStage;
uniform int uLN;
uniform vec4 uLP[4];
uniform vec3 uLD[4];
uniform vec3 uLC[4];
uniform float uShadowIdx;
uniform vec3 uSH[9];
uniform float uHasSH;
uniform vec3 uAmbient;
uniform float uExposure;
uniform float uRot;
vec3 shIrr(vec3 n) {
  float c = cos(uRot);
  float s = sin(uRot);
  vec3 m = vec3(c * n.x + s * n.z, n.y, -s * n.x + c * n.z);
  vec3 r = uSH[0] * 0.282095;
  r += uSH[1] * 0.488603 * m.y + uSH[2] * 0.488603 * m.z + uSH[3] * 0.488603 * m.x;
  r += uSH[4] * 1.092548 * m.x * m.y + uSH[5] * 1.092548 * m.y * m.z + uSH[6] * 0.315392 * (3.0 * m.z * m.z - 1.0);
  r += uSH[7] * 1.092548 * m.x * m.z + uSH[8] * 0.546274 * (m.x * m.x - m.y * m.y);
  return max(r, vec3(0.0));
}
uniform mat4 uView;
uniform vec3 uEye;
uniform vec3 uFwd;
uniform float uOrtho;
uniform vec3 uFront;
uniform vec3 uBack;
uniform float uRough;
uniform float uSheen;
uniform vec4 uTint;
uniform vec4 uState;
uniform float uUseMap;
uniform sampler2D uMap;
uniform vec4 uMapXf;
uniform vec2 uMapOff;
out vec4 outColor;
void main() {
  bool front = gl_FrontFacing;
  vec3 n = normalize(vN);
  if (!front) n = -n;
  vec3 v = uOrtho > 0.5 ? -uFwd : normalize(uEye - vW);
  vec3 base = front ? uFront : uBack;
  if (uUseMap > 0.5) {
    vec2 p = vUv - uMapOff;
    vec2 uv = vec2(uMapXf.x * p.x + uMapXf.y * p.y, -uMapXf.y * p.x + uMapXf.x * p.y);
    base *= pow(texture(uMap, uv).rgb, vec3(2.2));
  }
  if (uMapOn > 0.5) {
    float tt = clamp((vS - uRange.x) / max(uRange.y - uRange.x, 1e-9), 0.0, 1.0);
    base = vS < -1e20 ? vec3(0.18) : pow(texture(uRamp, vec2(tt, 0.5)).rgb, vec3(2.2)) * (front ? 1.0 : 0.8);
  }
  vec3 nv = mat3(uView) * n;
  vec3 vv = mat3(uView) * v;
  vec3 key = normalize(vec3(-0.45, 0.65, 0.62));
  vec3 fill = normalize(vec3(0.75, -0.25, 0.45));
  vec3 rim = normalize(vec3(0.1, 0.3, -0.9));
  float kd = clamp((dot(nv, key) + 0.35) / 1.35, 0.0, 1.0);
  float fd = clamp((dot(nv, fill) + 0.2) / 1.2, 0.0, 1.0);
  float rd = clamp(dot(nv, rim), 0.0, 1.0);
  vec3 amb = mix(vec3(0.075, 0.07, 0.07), vec3(0.2, 0.21, 0.24), 0.5 + 0.5 * nv.y);
  vec3 h = normalize(key + vv);
  float shin = mix(80.0, 8.0, uRough);
  float spec = pow(max(dot(nv, h), 0.0), shin) * (1.0 - uRough) * 0.35;
  float fres = pow(1.0 - clamp(dot(nv, vv), 0.0, 1.0), 3.0);
  vec3 col = base * (amb + kd * 0.95 * vec3(1.0, 0.97, 0.92) + fd * 0.28 * vec3(0.65, 0.75, 1.0) + rd * 0.12) + vec3(spec) + base * fres * uSheen * 0.9;
  if (uStage > 0.5) {
    vec3 ambient = uHasSH > 0.5 ? shIrr(n) : uAmbient;
    vec3 acc = vec3(0.0);
    for (int i = 0; i < 4; i++) {
      if (i >= uLN) break;
      int ty = int(uLP[i].w + 0.5);
      vec3 L;
      float att = 1.0;
      float wrap = 0.0;
      if (ty == 0) {
        L = normalize(-uLD[i]);
      } else {
        vec3 d = uLP[i].xyz - vW;
        float len = max(length(d), 1.0);
        L = d / len;
        att = min(pow(1500.0 / max(len, 50.0), 2.0), 4.0);
        if (ty == 2) att *= smoothstep(0.64, 0.84, dot(-L, normalize(uLD[i])));
        if (ty == 3) wrap = 0.5;
      }
      float nl = clamp((dot(n, L) + wrap) / (1.0 + wrap), 0.0, 1.0);
      float sh = 1.0;
      if (float(i) == uShadowIdx && nl > 0.0) sh = shadowAt(vW, n);
      vec3 hh = normalize(L + v);
      float sp = pow(max(dot(n, hh), 0.0), shin) * (1.0 - uRough) * 0.35 * step(0.001, nl);
      acc += uLC[i] * att * (base * nl + vec3(sp)) * sh;
    }
    col = base * ambient + acc + base * ambient * fres * uSheen * 0.9;
    col *= uExposure;
    col = col / (1.0 + col * 0.18);
  }
  if (uMapOn > 0.5) col = mix(base, col, 0.4);
  col = mix(col, uState.rgb, uState.a);
  col = mix(col, uTint.rgb, uTint.a);
  outColor = vec4(pow(col, vec3(1.0 / 2.2)), 1.0);
}`;

const LINE_VS = `#version 300 es
layout(location = 0) in vec3 aPos;
uniform mat4 uVP;
uniform mat4 uModel;
void main() {
  gl_Position = uVP * (uModel * vec4(aPos, 1.0));
}`;

const LINE_FS = `#version 300 es
precision highp float;
uniform vec4 uColor;
out vec4 outColor;
void main() {
  outColor = uColor;
}`;

function compile(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader {
  const sh = gl.createShader(type) as WebGLShader;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(sh) ?? 'shader');
  return sh;
}

function link(gl: WebGL2RenderingContext, vs: string, fs: string): WebGLProgram {
  const p = gl.createProgram() as WebGLProgram;
  gl.attachShader(p, compile(gl, gl.VERTEX_SHADER, vs));
  gl.attachShader(p, compile(gl, gl.FRAGMENT_SHADER, fs));
  gl.linkProgram(p);
  if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p) ?? 'link');
  return p;
}

export interface DrawPalette {
  accent: Float32Array;
  accentLin: Float32Array;
  line: Float32Array;
  bgLin: Float32Array;
}

interface MeshU {
  vp: WebGLUniformLocation | null;
  model: WebGLUniformLocation | null;
  view: WebGLUniformLocation | null;
  eye: WebGLUniformLocation | null;
  fwd: WebGLUniformLocation | null;
  ortho: WebGLUniformLocation | null;
  front: WebGLUniformLocation | null;
  back: WebGLUniformLocation | null;
  rough: WebGLUniformLocation | null;
  sheen: WebGLUniformLocation | null;
  tint: WebGLUniformLocation | null;
  state: WebGLUniformLocation | null;
  useMap: WebGLUniformLocation | null;
  map: WebGLUniformLocation | null;
  mapXf: WebGLUniformLocation | null;
  mapOff: WebGLUniformLocation | null;
  mapOn: WebGLUniformLocation | null;
  range: WebGLUniformLocation | null;
  ramp: WebGLUniformLocation | null;
  stage: WebGLUniformLocation | null;
  ln: WebGLUniformLocation | null;
  lp: WebGLUniformLocation | null;
  ld: WebGLUniformLocation | null;
  lc: WebGLUniformLocation | null;
  shadowIdx: WebGLUniformLocation | null;
  shadow: WebGLUniformLocation | null;
  shadowM: WebGLUniformLocation | null;
  shadowTexel: WebGLUniformLocation | null;
  sh: WebGLUniformLocation | null;
  hasSH: WebGLUniformLocation | null;
  ambient: WebGLUniformLocation | null;
  exposure: WebGLUniformLocation | null;
  rot: WebGLUniformLocation | null;
}

export class MeshRenderer {
  private readonly mesh: WebGLProgram;
  private readonly line: WebGLProgram;
  private readonly um: MeshU;
  private readonly lvp: WebGLUniformLocation | null;
  private readonly lmodel: WebGLUniformLocation | null;
  private readonly lcolor: WebGLUniformLocation | null;
  private readonly white: WebGLTexture | null;
  private readonly dummyDepth: WebGLTexture | null;
  private readonly textures = new Map<string, WebGLTexture | null>();

  constructor(private readonly gl: WebGL2RenderingContext, private readonly onTexture: () => void) {
    this.mesh = link(gl, MESH_VS, MESH_FS);
    this.line = link(gl, LINE_VS, LINE_FS);
    const u = (n: string): WebGLUniformLocation | null => gl.getUniformLocation(this.mesh, n);
    this.um = {
      vp: u('uVP'), model: u('uModel'), view: u('uView'), eye: u('uEye'), fwd: u('uFwd'), ortho: u('uOrtho'),
      front: u('uFront'), back: u('uBack'), rough: u('uRough'), sheen: u('uSheen'), tint: u('uTint'), state: u('uState'),
      useMap: u('uUseMap'), map: u('uMap'), mapXf: u('uMapXf'), mapOff: u('uMapOff'),
      mapOn: u('uMapOn'), range: u('uRange'), ramp: u('uRamp'), stage: u('uStage'), ln: u('uLN'), lp: u('uLP[0]'), ld: u('uLD[0]'), lc: u('uLC[0]'),
      shadowIdx: u('uShadowIdx'), shadow: u('uShadow'), shadowM: u('uShadowM'), shadowTexel: u('uShadowTexel'), sh: u('uSH[0]'), hasSH: u('uHasSH'),
      ambient: u('uAmbient'), exposure: u('uExposure'), rot: u('uRot'),
    };
    this.dummyDepth = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, this.dummyDepth);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.DEPTH_COMPONENT24, 1, 1, 0, gl.DEPTH_COMPONENT, gl.UNSIGNED_INT, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    this.lvp = gl.getUniformLocation(this.line, 'uVP');
    this.lmodel = gl.getUniformLocation(this.line, 'uModel');
    this.lcolor = gl.getUniformLocation(this.line, 'uColor');
    this.white = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, this.white);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array([255, 255, 255, 255]));
    gl.bindTexture(gl.TEXTURE_2D, null);
  }

  private texture(url: string): WebGLTexture | null {
    if (this.textures.has(url)) return this.textures.get(url) ?? null;
    this.textures.set(url, null);
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
      const gl = this.gl;
      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, img);
      gl.generateMipmap(gl.TEXTURE_2D);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
      gl.bindTexture(gl.TEXTURE_2D, null);
      this.textures.set(url, tex);
      this.onTexture();
    };
    img.src = url;
    return null;
  }

  draw(f: Frame, scene: Scene, style: ViewStyle, sel: Set<number>, hover: number, pal: DrawPalette, fx: Fx): void {
    const gl = this.gl;
    const list = scene.list;
    const faces = style !== 'mesh';
    const wire = style === 'shaded-mesh' || style === 'mesh';
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(true);
    gl.disable(gl.BLEND);
    gl.disable(gl.CULL_FACE);
    gl.enable(gl.POLYGON_OFFSET_FILL);
    gl.polygonOffset(wire ? 1 : 0, wire ? 1 : 0);
    const um = this.um;
    gl.useProgram(this.mesh);
    gl.uniformMatrix4fv(um.vp, false, f.vp);
    gl.uniformMatrix4fv(um.view, false, f.view);
    gl.uniform3f(um.eye, f.eye[0], f.eye[1], f.eye[2]);
    gl.uniform3f(um.fwd, f.fwd[0], f.fwd[1], f.fwd[2]);
    gl.uniform1f(um.ortho, f.ortho ? 1 : 0);
    gl.uniform1i(um.map, 0);
    const mp = fx.map;
    const st = fx.stage;
    gl.uniform1i(um.ramp, 1);
    gl.uniform1i(um.shadow, 2);
    gl.uniform2f(um.range, mp.min, mp.max);
    gl.uniform1f(um.stage, st.on ? 1 : 0);
    if (st.on) {
      gl.uniform1i(um.ln, st.count);
      gl.uniform4fv(um.lp, st.pos);
      gl.uniform3fv(um.ld, st.dir);
      gl.uniform3fv(um.lc, st.col);
      gl.uniform1f(um.shadowIdx, st.shadowIdx);
      gl.uniformMatrix4fv(um.shadowM, false, st.shadowMat);
      gl.uniform1f(um.shadowTexel, st.shadowTexel);
      gl.uniform3fv(um.sh, st.sh);
      gl.uniform1f(um.hasSH, st.hasSH ? 1 : 0);
      gl.uniform3f(um.ambient, st.ambient[0], st.ambient[1], st.ambient[2]);
      gl.uniform1f(um.exposure, st.exposure);
      gl.uniform1f(um.rot, st.rotation);
    }
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, mp.ramp ?? this.white);
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, st.on && st.shadowIdx >= 0 && st.shadowTex ? st.shadowTex : this.dummyDepth);
    gl.activeTexture(gl.TEXTURE0);
    for (let i = 0; i < list.length; i++) {
      const b = list[i];
      if (!b.ready) continue;
      const body = b.kind === 'body';
      const show = faces || body;
      gl.colorMask(show, show, show, show);
      gl.uniformMatrix4fv(um.model, false, b.model);
      const painted = style === 'panels' && !body;
      const fr = painted ? b.hue : b.front;
      const bk = painted ? b.hue : b.back;
      gl.uniform3f(um.front, fr[0], fr[1], fr[2]);
      if (painted) gl.uniform3f(um.back, bk[0] * 0.6, bk[1] * 0.6, bk[2] * 0.6);
      else gl.uniform3f(um.back, bk[0], bk[1], bk[2]);
      gl.uniform1f(um.rough, b.rough);
      gl.uniform1f(um.sheen, b.sheen);
      if (b.stateKind === 1) gl.uniform4f(um.state, 0.16, 0.23, 0.36, 0.55);
      else if (b.stateKind === 2) gl.uniform4f(um.state, pal.bgLin[0], pal.bgLin[1], pal.bgLin[2], 0.72);
      else gl.uniform4f(um.state, 0, 0, 0, 0);
      const selected = sel.has(b.id);
      if (selected) gl.uniform4f(um.tint, pal.accentLin[0], pal.accentLin[1], pal.accentLin[2], 0.3);
      else if (b.id === hover) gl.uniform4f(um.tint, 1, 1, 1, 0.14);
      else gl.uniform4f(um.tint, 0, 0, 0, 0);
      const tex = !painted && b.map ? this.texture(b.map) : null;
      gl.uniform1f(um.useMap, tex ? 1 : 0);
      gl.uniform1f(um.mapOn, mp.on && b.scalOn ? 1 : 0);
      gl.bindTexture(gl.TEXTURE_2D, tex ?? this.white);
      if (tex) {
        gl.uniform4f(um.mapXf, b.mapXf[0], b.mapXf[1], 0, 0);
        gl.uniform2f(um.mapOff, b.mapOff[0], b.mapOff[1]);
      }
      gl.bindVertexArray(b.vao);
      gl.drawElements(gl.TRIANGLES, b.tcount * 3, gl.UNSIGNED_INT, 0);
    }
    gl.colorMask(true, true, true, true);
    gl.disable(gl.POLYGON_OFFSET_FILL);
    if (wire) {
      gl.useProgram(this.line);
      gl.uniformMatrix4fv(this.lvp, false, f.vp);
      gl.enable(gl.BLEND);
      gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
      gl.depthMask(false);
      const alpha = style === 'mesh' ? 0.85 : 0.4;
      for (let i = 0; i < list.length; i++) {
        const b = list[i];
        if (!b.ready || b.ecount === 0) continue;
        gl.uniformMatrix4fv(this.lmodel, false, b.model);
        if (sel.has(b.id)) gl.uniform4f(this.lcolor, pal.accent[0], pal.accent[1], pal.accent[2], 0.95);
        else gl.uniform4f(this.lcolor, pal.line[0], pal.line[1], pal.line[2], alpha);
        gl.bindVertexArray(b.vaoWire);
        gl.drawElements(gl.LINES, b.ecount * 2, gl.UNSIGNED_INT, 0);
      }
      gl.depthMask(true);
      gl.disable(gl.BLEND);
    }
    gl.bindVertexArray(null);
    gl.bindTexture(gl.TEXTURE_2D, null);
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, null);
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, null);
    gl.activeTexture(gl.TEXTURE0);
    gl.disable(gl.DEPTH_TEST);
  }
}
