use crate::model::GpuModel;

const BLOCK: u32 = 256;
const WG: u32 = 64;
const PASSES: u32 = 2;

pub(crate) struct SelfContact {
    zero: wgpu::ComputePipeline,
    count: wgpu::ComputePipeline,
    scan_local: wgpu::ComputePipeline,
    scan_blocks: wgpu::ComputePipeline,
    scan_add: wgpu::ComputePipeline,
    fill: wgpu::ComputePipeline,
    vt: wgpu::ComputePipeline,
    apply: wgpu::ComputePipeline,
    bg: wgpu::BindGroup,
    _bufs: Vec<wgpu::Buffer>,
    buckets: u32,
    ntri: u32,
    cap: u32,
    n: u32,
}

fn entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn init_buffer<T: bytemuck::Pod>(device: &wgpu::Device, queue: &wgpu::Queue, label: &str, data: &[T]) -> wgpu::Buffer {
    let bytes: &[u8] = bytemuck::cast_slice(data);
    let buf = zero_buffer(device, label, bytes.len() as u64);
    queue.write_buffer(&buf, 0, bytes);
    buf
}

fn zero_buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size: size.max(16), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

impl SelfContact {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue, vs: &wgpu::Buffer, vinfo: &wgpu::Buffer, bgl1: &wgpu::BindGroupLayout, tris: &[[u32; 3]], model: &GpuModel) -> SelfContact {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("contact"), source: wgpu::ShaderSource::Wgsl(include_str!("contact.wgsl").into()) });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("contact0"),
            entries: &[entry(0, false), entry(1, true), entry(2, true), entry(3, true), entry(4, true), entry(5, false), entry(6, false), entry(7, false)],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("contact"), bind_group_layouts: &[Some(&bgl), Some(bgl1)], immediate_size: 0 });
        let make = |name: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(name),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let n = model.n;
        let ntri = tris.len() as u32;
        let want = (ntri.max(1) * 2).next_power_of_two();
        let buckets = want.clamp(BLOCK, 1 << 20);
        let cap = ntri * 32 + 1024;
        let flat: Vec<u32> = tris.iter().flat_map(|t| t.iter().copied()).collect();
        let mut starts = Vec::with_capacity(n);
        let mut list: Vec<u32> = Vec::new();
        for nb in model.neighbors.iter() {
            starts.push(list.len() as u32);
            list.extend_from_slice(nb);
        }
        let vdata: Vec<[u32; 4]> = (0..n).map(|i| [model.layer[i] as u32, model.thickness[i].to_bits(), starts[i], model.neighbors[i].len() as u32]).collect();
        let t_buf = init_buffer(device, queue, "sctri", &flat);
        let v_buf = init_buffer(device, queue, "scv", &vdata);
        let n_buf = init_buffer(device, queue, "scn", if list.is_empty() { &[0u32] } else { &list });
        let hmeta = zero_buffer(device, "hmeta", (2 * buckets as u64 + (buckets / BLOCK) as u64) * 4);
        let hent = zero_buffer(device, "hent", cap as u64 * 4);
        let disp = zero_buffer(device, "disp", n as u64 * 16);
        fn e(binding: u32, b: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry { binding, resource: b.as_entire_binding() }
        }
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("contact0"), layout: &bgl, entries: &[e(0, vs), e(1, vinfo), e(2, &t_buf), e(3, &v_buf), e(4, &n_buf), e(5, &hmeta), e(6, &hent), e(7, &disp)] });
        SelfContact {
            zero: make("sc_zero"),
            count: make("sc_count"),
            scan_local: make("sc_scan_local"),
            scan_blocks: make("sc_scan_blocks"),
            scan_add: make("sc_scan_add"),
            fill: make("sc_fill"),
            vt: make("sc_vt"),
            apply: make("sc_apply"),
            bg,
            _bufs: vec![t_buf, v_buf, n_buf, hmeta, hent, disp],
            buckets,
            ntri,
            cap,
            n: n as u32,
        }
    }

    pub(crate) fn uniform(&self) -> [u32; 4] {
        [self.buckets, self.ntri, self.cap, 0]
    }

    pub(crate) fn encode(&self, pass: &mut wgpu::ComputePass<'_>, restore: &wgpu::BindGroup) {
        let groups = |count: u32| count.div_ceil(WG).max(1);
        pass.set_bind_group(0, &self.bg, &[]);
        pass.set_pipeline(&self.zero);
        pass.dispatch_workgroups(groups(self.buckets), 1, 1);
        pass.set_pipeline(&self.count);
        pass.dispatch_workgroups(groups(self.ntri), 1, 1);
        pass.set_pipeline(&self.scan_local);
        pass.dispatch_workgroups(groups(self.buckets / BLOCK), 1, 1);
        pass.set_pipeline(&self.scan_blocks);
        pass.dispatch_workgroups(1, 1, 1);
        pass.set_pipeline(&self.scan_add);
        pass.dispatch_workgroups(groups(self.buckets), 1, 1);
        pass.set_pipeline(&self.zero);
        pass.dispatch_workgroups(groups(self.buckets), 1, 1);
        pass.set_pipeline(&self.fill);
        pass.dispatch_workgroups(groups(self.ntri), 1, 1);
        for _ in 0..PASSES {
            pass.set_pipeline(&self.vt);
            pass.dispatch_workgroups(groups(self.n), 1, 1);
            pass.set_pipeline(&self.apply);
            pass.dispatch_workgroups(groups(self.n), 1, 1);
        }
        pass.set_bind_group(0, restore, &[]);
    }
}
