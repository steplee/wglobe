use nalgebra::{Isometry3,Point3,Vector3};
use wgpu::util::DeviceExt;

use super::AppObjects;

type Isometry3f = Isometry3<f32>;
type Point3f = Point3<f32>;
type Vector3f = Vector3<f32>;

pub struct CameraIntrin {
    tlbr: [f32; 4],
    zn: f32,
    zf: f32,
}

pub struct CameraPose {
    pub intrin: CameraIntrin,
    pub pose: Isometry3f,
}

pub struct Scene {
    pub cam: CameraPose,
    pub time: f32,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl CameraIntrin {
    fn to_matrix(&self) -> nalgebra::Matrix4<f32> {
        let mut out = nalgebra::Matrix4::<f32>::identity();

        log::warn!("proj is identity for now");
        return out;

        let left = self.tlbr[0];
        let rght = self.tlbr[2];
        let top = self.tlbr[1];
        let bot = self.tlbr[3];
        out[0*4+0] = 2.0*self.zn / (rght-left);
        out[1*4+1] = 2.0*self.zn / (top-bot);
        out[0*4+2] = (rght+left)/(rght-left);
        out[1*4+2] = (top+bot)/(top-bot);
        out[2*4+2] = -(self.zn+self.zf)/(self.zf-self.zn);
        out[2*4+3] = 2.0*self.zf*self.zn/(self.zf-self.zn);
        out[3*4+2] = -1.0;
        return out;
    }
}

impl Default for CameraPose {
    fn default() -> Self {
        return CameraPose {
            intrin: CameraIntrin { tlbr: [-1.,-1.,1.,1.], zn: 0.001, zf: 100. },
            pose: Isometry3f::look_at_lh(&Point3f::new(0.,0.,-1.), &Vector3f::zeros().into(), &Vector3f::y())
            // pose: Isometry3f::identity(),
        }
    }
}

impl Scene {
    pub fn new(ao: &AppObjects) -> Self {
        let cam = Default::default();

        let buffer = ao.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cameraBuffer"),
            contents: bytemuck::cast_slice(&[LoweredScene::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let bind_group_layout = ao.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cameraBgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    // count: 1u32.try_into().ok(),
                    count: None,
                },
            ],
        });

        let bind_group = ao.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cameraBg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Scene {
            cam,
            time: 0.,
            buffer,
            bind_group_layout,
            bind_group
        }
    }

    pub fn update_buffer(&self, ao: &AppObjects) {
        let lowered_scene: LoweredScene = self.into();
        ao.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&lowered_scene));
    }
}

fn slice_to_array<T, const N: usize>(s: &[T]) -> [T; N] 
where T: Default + Copy
{
    let mut out: [T; N] = [T::default(); N];
    out.copy_from_slice(s);
    return out;
}

impl Into<LoweredScene> for &Scene {
    fn into(self) -> LoweredScene {
        log::info!("mv:\n{}", self.cam.pose.to_matrix());
        LoweredScene {
            mv: slice_to_array(self.cam.pose.to_matrix().as_slice()),
            proj: slice_to_array(self.cam.intrin.to_matrix().as_slice()),
            time: self.time,
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug, Default)]
pub struct LoweredScene {
    mv: [f32; 16],
    proj: [f32; 16],

    time: f32,
    pad1: f32,
    pad2: f32,
    pad3: f32,
}

#[test]
fn check_bytemuck_stuff() {
    let mut lscene = LoweredScene::default();
    lscene.mv[0] = 1.0;
    lscene.mv[15] = 15.0;

    println!("lscene: {lscene:?}");


    // little exercise to remind myself of some pointer stuff. Probably how bytemuck::bytes_of
    // works.
    fn my_bytes_of<T>(t: &T) -> &[u8] {
        unsafe {
            let tt : *const T = t;
            let ttt = tt as (*const u8);
            return &*std::ptr::slice_from_raw_parts(ttt, std::mem::size_of::<T>());
        }
    }

    println!("lscene bytes: {:?}", bytemuck::bytes_of(&lscene));
    println!("lscene bytes: {:?}", my_bytes_of(&lscene));
}

