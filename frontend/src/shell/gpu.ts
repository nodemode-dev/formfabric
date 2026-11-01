export async function probeGpu(): Promise<string> {
  try {
    const gpu = (navigator as unknown as { gpu?: { requestAdapter(): Promise<unknown> } }).gpu;
    if (gpu && (await gpu.requestAdapter())) return 'WebGPU';
  } catch { /* fall through to WebGL */ }
  try {
    const c = document.createElement('canvas');
    if (c.getContext('webgl2')) return 'WebGL2';
    if (c.getContext('webgl')) return 'WebGL';
  } catch { /* none */ }
  return 'none';
}
