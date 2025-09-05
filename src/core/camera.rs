use nalgebra::{Isometry3,Point3,Vector3};

type Isometry3f = Isometry3<f32>;
type Point3f = Point3<f32>;
type Vector3f = Vector3<f32>;


pub struct CameraIntrin {
    tlbr: [f32; 4],
    zn: f32,
    zf: f32,
}


pub struct Camera {
    pub intrin: CameraIntrin,
    pub pose: Isometry3f,
}

impl Default for Camera {
    fn default() -> Self {
        return Camera {
            intrin: CameraIntrin { tlbr: [-1.,-1.,1.,1.], zn: 0.001, zf: 100. },
            pose: Isometry3f::look_at_lh(&Point3f::new(0.,0.,-2.), &Vector3f::zeros().into(), &Vector3f::y())
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug, Default)]
pub struct LoweredScene {
    mv: [f32; 16],
    proj: [f32; 16],
    mvp: [f32; 16],
    time: f32,
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

