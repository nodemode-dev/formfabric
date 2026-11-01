use std::fmt;
use std::sync::mpsc;

use bytemuck::{Pod, Zeroable};
use ffb_sim::{Cloth, Collider, Params, PinPoint, StepStats};

use crate::colliders::{pack, Packed};
use crate::model::GpuModel;

const STRIDE: u64 = 256;
const WG: u32 = 64;

#[derive(Debug, Clone)]
pub struct GpuError(pub String);

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for GpuError {}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct VState {
    x: [f32; 4],
    xn: [f32; 4],
    y: [f32; 4],
    v: [f32; 4],
    a: [f32; 4],
    e: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct VInfo {
    m: [f32; 4],
    pa: [f32; 4],
    pb: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SubU {
    gh: [f32; 4],
    p0: [f32; 4],
    p1: [f32; 4],
    tg: [f32; 4],
    u0: [u32; 4],
    u1: [u32; 4],
    u2: [u32; 4],
}

#[derive(Debug, Clone, Copy)]
struct PinState {
    vertex: u32,
    prev: [f32; 3],
    target: [f32; 3],
}

pub(crate) fn request_adapter() -> Option<(wgpu::Instance, wgpu::Adapter)> {
    let instance = wgpu::Instance::default();
    for fallback in [false, true] {
        let opts = wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: fallback, compatible_surface: None, apply_limit_buckets: false };
        if let Ok(adapter) = pollster::block_on(instance.request_adapter(&opts)) {
            return Some((instance, adapter));
        }
    }
    None
}

struct Pipelines {
    begin_a: wgpu::ComputePipeline,
    begin_b: wgpu::ComputePipeline,
    solve: wgpu::ComputePipeline,
    finish: wgpu::ComputePipeline,
    bgl0: wgpu::BindGroupLayout,
    bgl1: wgpu::BindGroupLayout,
}

pub struct GpuSolver {
    _instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipes: Pipelines,
    params: Params,
    model: GpuModel,
    n: usize,
    vs: wgpu::Buffer,
    vinfo: wgpu::Buffer,
    elem: wgpu::Buffer,
    idx: wgpu::Buffer,
    col: wgpu::Buffer,
    cidx: wgpu::Buffer,
    outb: wgpu::Buffer,
    staging: wgpu::Buffer,
    sub_u: wgpu::Buffer,
    sub_cap: u32,
    col_u: wgpu::Buffer,
    bg0: wgpu::BindGroup,
    bg1: wgpu::BindGroup,
    packed: Packed,
    infos: Vec<VInfo>,
    pins: Vec<PinState>,
    grab: Option<(u32, [f32; 3])>,
    out: Vec<[f32; 3]>,
    info_dirty: bool,
    adapter_name: String,
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: None },
        count: None,
    }
}

fn make_pipelines(device: &wgpu::Device) -> Pipelines {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("vbd"), source: wgpu::ShaderSource::Wgsl(include_str!("vbd.wgsl").into()) });
    let bgl0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("vbd0"),
        entries: &[storage_entry(0, false), storage_entry(1, true), storage_entry(2, true), storage_entry(3, true), storage_entry(4, true), storage_entry(5, true), storage_entry(6, false)],
    });
    let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("vbd1"), entries: &[uniform_entry(0), uniform_entry(1)] });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("vbd"), bind_group_layouts: &[Some(&bgl0), Some(&bgl1)], immediate_size: 0 });
    let make = |entry: &str| {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(entry),
            layout: Some(&layout),
            module: &module,
            entry_point: Some(entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };
    Pipelines { begin_a: make("begin_a"), begin_b: make("begin_b"), solve: make("solve"), finish: make("finish"), bgl0, bgl1 }
}

fn storage_buffer(device: &wgpu::Device, label: &str, size: u64, extra: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: size.max(16), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | extra, mapped_at_creation: false })
}

fn uniform_buffer(device: &wgpu::Device, label: &str, slots: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: slots.max(1) * STRIDE, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

impl GpuSolver {
    pub fn new(cloth: Cloth, params: Params) -> Result<GpuSolver, GpuError> {
        let (instance, adapter) = request_adapter().ok_or_else(|| GpuError("no GPU adapter".into()))?;
        let adapter_name = adapter.get_info().name;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("ffb-gpu"), ..Default::default() })).map_err(|e| GpuError(format!("device: {e}")))?;
        let pipes = make_pipelines(&device);
        let model = GpuModel::build(&cloth);
        let n = cloth.positions.len();
        if n == 0 {
            return Err(GpuError("empty cloth".into()));
        }
        let vs = storage_buffer(&device, "vs", n as u64 * 96, wgpu::BufferUsages::empty());
        let vinfo = storage_buffer(&device, "vinfo", n as u64 * 48, wgpu::BufferUsages::empty());
        let elem = storage_buffer(&device, "elem", model.elem.len() as u64 * 16, wgpu::BufferUsages::empty());
        let idx = storage_buffer(&device, "idx", model.idx.len() as u64 * 4, wgpu::BufferUsages::empty());
        let outb = storage_buffer(&device, "out", n as u64 * 32, wgpu::BufferUsages::COPY_SRC);
        let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("staging"), size: n as u64 * 32, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&elem, 0, bytemuck::cast_slice(&model.elem));
        queue.write_buffer(&idx, 0, bytemuck::cast_slice(&model.idx));
        let packed = pack(&[]);
        let col = storage_buffer(&device, "col", packed.col.len() as u64 * 16, wgpu::BufferUsages::empty());
        let cidx = storage_buffer(&device, "cidx", packed.cidx.len() as u64 * 4, wgpu::BufferUsages::empty());
        let sub_cap = params.substeps.max(1);
        let sub_u = uniform_buffer(&device, "sub", sub_cap as u64);
        let col_u = uniform_buffer(&device, "colour", model.colors.len() as u64);
        let mut cu = vec![0u8; model.colors.len().max(1) * STRIDE as usize];
        for (c, (start, count)) in model.colors.iter().enumerate() {
            let rec = [*start, *count, 0, 0];
            cu[c * STRIDE as usize..c * STRIDE as usize + 16].copy_from_slice(bytemuck::cast_slice(&rec));
        }
        queue.write_buffer(&col_u, 0, &cu);
        let bg0 = Self::group0(&device, &pipes, &vs, &vinfo, &elem, &idx, &col, &cidx, &outb);
        let bg1 = Self::group1(&device, &pipes, &sub_u, &col_u);
        let mut s = GpuSolver {
            _instance: instance,
            device,
            queue,
            pipes,
            params,
            infos: Vec::new(),
            pins: Vec::new(),
            grab: None,
            out: cloth.positions.clone(),
            info_dirty: true,
            adapter_name,
            model,
            n,
            vs,
            vinfo,
            elem,
            idx,
            col,
            cidx,
            outb,
            staging,
            sub_u,
            sub_cap,
            col_u,
            bg0,
            bg1,
            packed,
        };
        s.infos = (0..n).map(|i| VInfo { m: [s.model.mass[i], s.model.friction[i], 0.0, 0.0], pa: [0.0; 4], pb: [0.0; 4] }).collect();
        s.upload_colliders();
        let start = cloth.positions.clone();
        s.write_state(&start);
        s.set_pins(cloth.pins);
        Ok(s)
    }

    #[allow(clippy::too_many_arguments)]
    fn group0(device: &wgpu::Device, pipes: &Pipelines, vs: &wgpu::Buffer, vinfo: &wgpu::Buffer, elem: &wgpu::Buffer, idx: &wgpu::Buffer, col: &wgpu::Buffer, cidx: &wgpu::Buffer, outb: &wgpu::Buffer) -> wgpu::BindGroup {
        fn e(binding: u32, b: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry { binding, resource: b.as_entire_binding() }
        }
        device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("g0"), layout: &pipes.bgl0, entries: &[e(0, vs), e(1, vinfo), e(2, elem), e(3, idx), e(4, col), e(5, cidx), e(6, outb)] })
    }

    fn group1(device: &wgpu::Device, pipes: &Pipelines, sub_u: &wgpu::Buffer, col_u: &wgpu::Buffer) -> wgpu::BindGroup {
        fn e(binding: u32, b: &wgpu::Buffer, size: u64) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry { binding, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: b, offset: 0, size: wgpu::BufferSize::new(size) }) }
        }
        device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("g1"), layout: &pipes.bgl1, entries: &[e(0, sub_u, std::mem::size_of::<SubU>() as u64), e(1, col_u, 16)] })
    }

    fn upload_colliders(&mut self) {
        self.col = storage_buffer(&self.device, "col", self.packed.col.len() as u64 * 16, wgpu::BufferUsages::empty());
        self.cidx = storage_buffer(&self.device, "cidx", self.packed.cidx.len() as u64 * 4, wgpu::BufferUsages::empty());
        self.queue.write_buffer(&self.col, 0, bytemuck::cast_slice(&self.packed.col));
        self.queue.write_buffer(&self.cidx, 0, bytemuck::cast_slice(&self.packed.cidx));
        self.bg0 = Self::group0(&self.device, &self.pipes, &self.vs, &self.vinfo, &self.elem, &self.idx, &self.col, &self.cidx, &self.outb);
    }

    fn write_state(&mut self, p: &[[f32; 3]]) {
        let states: Vec<VState> = (0..self.n)
            .map(|i| {
                let q = p.get(i).copied().unwrap_or([0.0; 3]);
                let x = [q[0], q[1], q[2], 0.0];
                VState { x, xn: x, y: x, v: [0.0; 4], a: [0.0; 4], e: [0.0; 4] }
            })
            .collect();
        self.queue.write_buffer(&self.vs, 0, bytemuck::cast_slice(&states));
    }

    fn write_infos(&mut self) {
        for info in self.infos.iter_mut() {
            info.m[2] = 0.0;
        }
        for p in &self.pins {
            let info = &mut self.infos[p.vertex as usize];
            info.m[2] = 1.0;
            info.pa = [p.prev[0], p.prev[1], p.prev[2], 0.0];
            info.pb = [p.target[0], p.target[1], p.target[2], 0.0];
        }
        self.queue.write_buffer(&self.vinfo, 0, bytemuck::cast_slice(&self.infos));
        self.info_dirty = false;
    }

    pub fn set_colliders(&mut self, colliders: Vec<Collider>) {
        self.packed = pack(&colliders);
        self.upload_colliders();
    }

    pub fn set_pins(&mut self, pins: Vec<PinPoint>) {
        let old = std::mem::take(&mut self.pins);
        for p in pins {
            let i = p.vertex as usize;
            if i >= self.n {
                continue;
            }
            let prev = old.iter().find(|o| o.vertex == p.vertex).map_or(self.out[i], |o| o.target);
            self.pins.push(PinState { vertex: p.vertex, prev, target: p.target });
        }
        self.info_dirty = true;
    }

    pub fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    pub fn params(&self) -> Params {
        self.params
    }

    pub fn positions(&self) -> &[[f32; 3]] {
        &self.out
    }

    pub fn vertex_count(&self) -> usize {
        self.n
    }

    pub fn color_count(&self) -> usize {
        self.model.colors.len()
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn set_positions(&mut self, p: &[[f32; 3]]) {
        for (i, q) in p.iter().enumerate().take(self.n) {
            self.out[i] = *q;
        }
        let snapshot = self.out.clone();
        self.write_state(&snapshot);
        for p in self.pins.iter_mut() {
            p.prev = p.target;
        }
        self.info_dirty = true;
    }

    pub fn grab(&mut self, vertex: u32, target: Option<[f32; 3]>) {
        self.grab = match target {
            Some(t) if (vertex as usize) < self.n => Some((vertex, t)),
            _ => None,
        };
    }

    fn ensure_substeps(&mut self, substeps: u32) {
        if substeps > self.sub_cap {
            self.sub_cap = substeps;
            self.sub_u = uniform_buffer(&self.device, "sub", substeps as u64);
            self.bg1 = Self::group1(&self.device, &self.pipes, &self.sub_u, &self.col_u);
        }
    }

    pub fn step(&mut self) -> StepStats {
        let t0 = std::time::Instant::now();
        let substeps = self.params.substeps.max(1);
        let iters = self.params.iterations.max(1);
        self.ensure_substeps(substeps);
        if self.info_dirty {
            self.write_infos();
        }
        let h = self.params.time_step.max(1.0e-6) / substeps as f32;
        let g = self.params.gravity;
        let damp = 1.0 / (1.0 + self.params.air_damping.max(0.0) * 10.0 * h);
        let (gi, gt) = self.grab.map_or((u32::MAX, [0.0; 3]), |(v, t)| (v, t));
        let mut bytes = vec![0u8; substeps as usize * STRIDE as usize];
        for s in 0..substeps {
            let u = SubU {
                gh: [g[0], g[1], g[2], h],
                p0: [(s + 1) as f32 / substeps as f32, damp, self.params.skin_offset.max(0.0), self.params.friction.max(0.0)],
                p1: [if self.params.nonlinear { 1.0 } else { 0.0 }, self.params.stiffening, 0.0, 0.0],
                tg: [gt[0], gt[1], gt[2], 0.0],
                u0: [self.n as u32, self.packed.count, gi, 0],
                u1: [self.model.tri_off, self.model.hinge_off, self.model.link_off, 0],
                u2: [self.model.order_off, self.model.hdr_off, self.model.adj_off, 0],
            };
            let o = s as usize * STRIDE as usize;
            bytes[o..o + std::mem::size_of::<SubU>()].copy_from_slice(bytemuck::bytes_of(&u));
        }
        self.queue.write_buffer(&self.sub_u, 0, &bytes);

        let groups = (self.n as u32).div_ceil(WG);
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("vbd"), timestamp_writes: None });
            pass.set_bind_group(0, &self.bg0, &[]);
            for s in 0..substeps {
                let so = (s as u64 * STRIDE) as u32;
                pass.set_bind_group(1, &self.bg1, &[so, 0]);
                pass.set_pipeline(&self.pipes.begin_a);
                pass.dispatch_workgroups(groups, 1, 1);
                pass.set_pipeline(&self.pipes.begin_b);
                pass.dispatch_workgroups(groups, 1, 1);
                pass.set_pipeline(&self.pipes.solve);
                for _ in 0..iters {
                    for (c, (_, count)) in self.model.colors.iter().enumerate() {
                        pass.set_bind_group(1, &self.bg1, &[so, (c as u64 * STRIDE) as u32]);
                        pass.dispatch_workgroups(count.div_ceil(WG), 1, 1);
                    }
                }
                pass.set_bind_group(1, &self.bg1, &[so, 0]);
                pass.set_pipeline(&self.pipes.finish);
                pass.dispatch_workgroups(groups, 1, 1);
            }
        }
        enc.copy_buffer_to_buffer(&self.outb, 0, &self.staging, 0, self.n as u64 * 32);
        self.queue.submit(Some(enc.finish()));
        let (tx, rx) = mpsc::channel();
        self.staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        let mut vmax = 0.0f32;
        let mut contacts = 0u32;
        if matches!(rx.recv(), Ok(Ok(()))) {
            let view = self.staging.slice(..).get_mapped_range().map(|v| v.to_vec()).unwrap_or_default();
            let data: &[[f32; 4]] = bytemuck::cast_slice(&view);
            for i in 0..self.n {
                let p = data[2 * i];
                self.out[i] = [p[0], p[1], p[2]];
                vmax = vmax.max(p[3]);
                if data[2 * i + 1][0] > 0.5 {
                    contacts += 1;
                }
            }
            self.staging.unmap();
        }
        for p in self.pins.iter_mut() {
            p.prev = p.target;
        }
        if !self.pins.is_empty() {
            self.info_dirty = true;
        }
        StepStats { ms: t0.elapsed().as_secs_f32() * 1000.0, iterations: substeps * iters, max_velocity: vmax, contacts }
    }
}
