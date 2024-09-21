use std::num::FpCategory::Infinite;
use nalgebra::{min, Vector3};

pub fn triangle_aabb_intersection(vertices: &[Vector3<f32>; 3], aabb: &AABB) -> bool {
    let mut minX = f32::MAX;
    let mut minY = f32::MAX;
    let mut minZ = f32::MAX;
    let mut maxX = -f32::MAX;
    let mut maxY = -f32::MAX;
    let mut maxZ = -f32::MAX;

    for vertex in vertices {
        if vertex.x < minX {
            minX = vertex.x;
        }
        if vertex.y < minY {
            minY = vertex.y;
        }
        if vertex.z < minZ {
            minZ = vertex.z;
        }

        if vertex.x > maxX {
            maxX = vertex.x;
        }
        if vertex.y > maxY {
            maxY = vertex.y;
        }
        if vertex.z > maxZ {
            maxZ = vertex.z;
        }
    }

    if minX < aabb.min.x || minY < aabb.min.y || minZ < aabb.min.z {
        return false;
    }
    if maxX > aabb.max.x || maxY > aabb.max.y || maxZ > aabb.max.z {
        return false;
    }

    let box_axes = [
        Vector3::<f32>::from([1.0, 0.0, 0.0]),
        Vector3::<f32>::from([0.0, 1.0, 0.0]),
        Vector3::<f32>::from([0.0, 0.0, 1.0]),
    ];

    let v0 = &vertices[0];
    let v1 = &vertices[1];
    let v2 = &vertices[2];

    let f0 = (v1 - v0).normalize();
    let f1 = (v2 - v1).normalize();
    let f2 = (v0 - v2).normalize();

    let axes = [
        box_axes[0],
        box_axes[1],
        box_axes[2],
        f0.cross(&box_axes[0]),
        f0.cross(&box_axes[1]),
        f0.cross(&box_axes[2]),
        f1.cross(&box_axes[0]),
        f1.cross(&box_axes[1]),
        f1.cross(&box_axes[2]),
        f2.cross(&box_axes[0]),
        f2.cross(&box_axes[1]),
        f2.cross(&box_axes[2]),
        f0.cross(&f1).cross(&f2), // Triangle normal
    ];

    for axis in &axes {
        if !overlap_on_axis(vertices, aabb, axis) {
            return false;
        }
    }

    true
}

fn overlap_on_axis(tri: &[Vector3<f32>; 3], aabb: &AABB, axis: &Vector3<f32>) -> bool {
    let (tri_min, tri_max) = project_triangle_on_axis(tri, axis);
    let (aabb_min, aabb_max) = project_aabb_on_axis(aabb, axis);

    tri_max >= aabb_min && aabb_max >= tri_min
}

fn project_triangle_on_axis(tri: &[Vector3<f32>; 3], axis: &Vector3<f32>) -> (f32, f32) {
    let mut min = tri[0].dot(axis);
    let mut max = min;

    for vertex in [&tri[1], &tri[2]] {
        let projection = vertex.dot(axis);

        if projection < min { min = projection; }
        if projection > max { max = projection; }
    }

    (min, max)
}

fn project_aabb_on_axis(aabb: &AABB, axis: &Vector3<f32>) -> (f32, f32) {
    let vertices = [
        Vector3::<f32>::from([aabb.min.x, aabb.min.y, aabb.min.z]),
        Vector3::<f32>::from([aabb.min.x, aabb.min.y, aabb.max.z]),
        Vector3::<f32>::from([aabb.min.x, aabb.max.y, aabb.min.z]),
        Vector3::<f32>::from([aabb.min.x, aabb.max.y, aabb.max.z]),
        Vector3::<f32>::from([aabb.max.x, aabb.min.y, aabb.min.z]),
        Vector3::<f32>::from([aabb.max.x, aabb.min.y, aabb.max.z]),
        Vector3::<f32>::from([aabb.max.x, aabb.max.y, aabb.min.z]),
        Vector3::<f32>::from([aabb.max.x, aabb.max.y, aabb.max.z]),
    ];

    let mut min = vertices[0].dot(axis);
    let mut max = min;

    for vertex in &vertices[1..] {
        let projection = vertex.dot(axis);

        if projection < min { min = projection; }
        if projection > max { max = projection; }
    }

    (min, max)
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl AABB {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        AABB { min, max }
    }

    pub fn center(&self) -> Vector3<f32> {
        Vector3::<f32>::from([
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        ])
    }

    pub fn size(&self) -> Vector3<f32> {
        Vector3::<f32>::from([
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        ])
    }

    pub fn half_size(&self) -> Vector3<f32> {
        Vector3::<f32>::from([
            (self.max.x - self.min.x) / 2.0,
            (self.max.y - self.min.y) / 2.0,
            (self.max.z - self.min.z) / 2.0,
        ])
    }

    pub fn split(&self) -> [AABB; 8] {
        let mut boxes = [*self; 8];
        let mid = [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        ];

        for i in 0..8 {
            for j in 0..3 {
                if i & (1 << j) == 0 {
                    boxes[i].max[j] = mid[j];
                } else {
                    boxes[i].min[j] = mid[j];
                }
            }
        }
        boxes
    }
}