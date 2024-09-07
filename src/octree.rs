use nalgebra::{Vector3};
use crate::aabb::{triangle_aabb_intersection, AABB};


#[derive(Debug)]
pub struct OctreeNode {
    pub aabb: AABB,
    pub children: Option<[Box<OctreeNode>; 8]>,
    pub faces: Vec<usize>, // This contains the face indices
}

impl OctreeNode {
    pub fn new(bounding_box: AABB) -> Self {
        OctreeNode {
            aabb: bounding_box,
            children: None,
            faces: Vec::new(),
        }
    }

    pub fn insert(&mut self, face_index: usize, vertices: &[Vector3<f32>; 3], max_depth: usize, depth: usize) {
        // if depth > 3{
        //     println!("depth: {}", depth > 2);
        // }
        if depth >= max_depth {
            self.faces.push(face_index);
            return;
        }

        if self.children.is_none() {
            let bounding_boxes = self.aabb.split();

            // subdivide
            let children: [Box<OctreeNode>; 8] = [
                Box::new(OctreeNode::new(bounding_boxes[0])),
                Box::new(OctreeNode::new(bounding_boxes[1])),
                Box::new(OctreeNode::new(bounding_boxes[2])),
                Box::new(OctreeNode::new(bounding_boxes[3])),
                Box::new(OctreeNode::new(bounding_boxes[4])),
                Box::new(OctreeNode::new(bounding_boxes[5])),
                Box::new(OctreeNode::new(bounding_boxes[6])),
                Box::new(OctreeNode::new(bounding_boxes[7])),
            ];

            self.children = Some(children);
        }

        match self.children {
            None => {}
            Some(ref mut children) => {
                for child in children {
                    if triangle_aabb_intersection(&vertices, &child.aabb) {
                        child.insert(face_index, vertices, max_depth, depth + 1);
                    }
                }
            }
        }

        // for child in self.children.as_mut().unwrap().par_iter_mut() {
        //     if triangle_aabb_intersection(&vertices, &child.aabb) {
        //         child.insert(face_index, vertices, max_depth, depth + 1)
        //     }
        // }
    }
}
