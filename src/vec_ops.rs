use nalgebra::Vector3;

pub trait Vector3Ext {
    fn ceil(&self) -> Vector3<f32>;
    fn floor(&self) -> Vector3<f32>;
}

impl Vector3Ext for Vector3<f32> {
    fn ceil(&self) -> Vector3<f32> {
        Vector3::new(self.x.ceil(), self.y.ceil(), self.z.ceil())
    }

    fn floor(&self) -> Vector3<f32> {
        Vector3::new(self.x.floor(), self.y.floor(), self.z.floor())
    }
}