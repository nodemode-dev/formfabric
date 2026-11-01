import type { Frame } from './frame';
import type { ViewStyle } from './model';
import type { Scene } from './scene';

const MESH_VS = `#version 300 es
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNrm;
layout(location = 2) in vec2 aUv;
uniform mat4 uVP;
uniform mat4 uModel;
out vec3 vN;
out vec3 vW;
out vec2 vUv;
void main() {
  vec4 w = uModel * vec4(aPos, 1.0);
  vW = w.xyz;
  vN = mat3(uModel) * aNrm;
  vUv = aUv;
  gl_Position = uVP * w;
}`;

const MESH_FS = `#version 300 es
precision highp float;
in vec3 vN;
in vec3 vW;
in vec2 vUv;
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
}

export class MeshRenderer {
  private readonly mesh: WebGLProgram;
  private readonly line: WebGLProgram;
  private readonly um: MeshU;
  private readonly lvp: WebGLUniformLocation | null;
  private readonly lmodel: WebGLUniformLocation | null;
  private readonly lcolor: WebGLUniformLocation | null;
  private readonly white: WebGLTexture | null;
  private readonly textures = new Map<string, WebGLTexture | null>();

  constructor(private readonly gl: WebGL2RenderingContext, private readonly onTexture: () => void) {
    this.mesh = link(gl, MESH_VS, MESH_FS);
    this.line = link(gl, LINE_VS, LINE_FS);
    const u = (n: string): WebGLUniformLocation | null => gl.getUniformLocation(this.mesh, n);
    this.um = {
      vp: u('uVP'), model: u('uModel'), view: u('uView'), eye: u('uEye'), fwd: u('uFwd'), ortho: u('uOrtho'),
      front: u('uFront'), back: u('uBack'), rough: u('uRough'), sheen: u('uSheen'), tint: u('uTint'), state: u('uState'),
      useMap: u('uUseMap'), map: u('uMap'), mapXf: u('uMapXf'), mapOff: u('uMapOff'),
    };
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

  draw(f: Frame, scene: Scene, style: ViewStyle, sel: Set<number>, hover: number, pal: DrawPalette): void {
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
    gl.disable(gl.DEPTH_TEST);
  }
}
