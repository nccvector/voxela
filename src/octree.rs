use std::sync::{Arc, Mutex};
use nalgebra::{center, Vector3};
use rayon::prelude::*;
use crate::aabb::{triangle_aabb_intersection, AABB};
use crate::loader::Mesh;

#[derive(Debug, Clone)]
pub struct OctreeNode {
    pub aabb: AABB,
    pub children: Option<[Arc<Mutex<OctreeNode>>; 8]>,
    pub faces: Vec<usize>, // This contains the face indices
}

impl OctreeNode {
    pub fn new(aabb: AABB) -> Self {
        OctreeNode {
            aabb,
            children: None,
            faces: Vec::new(),
        }
    }

    // Function to subdivide the node into 8 children
    fn subdivide(&mut self) {
        let bounding_boxes = self.aabb.split();

        let children = [
            Arc::new(Mutex::new(OctreeNode::new(bounding_boxes[0].clone()))),
            Arc::new(Mutex::new(OctreeNode::new(bounding_boxes[1].clone()))),
            Arc::new(Mutex::new(OctreeNode::new(bounding_boxes[2].clone()))),
            Arc::new(Mutex::new(OctreeNode::new(bounding_boxes[3].clone()))),
            Arc::new(Mutex::new(OctreeNode::new(bounding_boxes[4].clone()))),
            Arc::new(Mutex::new(OctreeNode::new(bounding_boxes[5].clone()))),
            Arc::new(Mutex::new(OctreeNode::new(bounding_boxes[6].clone()))),
            Arc::new(Mutex::new(OctreeNode::new(bounding_boxes[7].clone()))),
        ];

        self.children = Some(children);
    }

    pub fn insert_face(&mut self, face_index: usize, &vertices: &[Vector3<f32>; 3], faceAABB: &AABB) {
        if triangle_aabb_intersection(&vertices, faceAABB, &self.aabb) {
            self.faces.push(face_index);
        }
    }
}

#[derive(Debug)]
pub struct Octree {
    pub root: Arc<Mutex<OctreeNode>>,
    pub leaves: Vec<Arc<Mutex<OctreeNode>>>,
}

impl Octree {
    pub fn new(mesh: &Mesh) -> Self {
        let bmin = mesh.aabb.min.min().floor() - 1.0;
        let mut bmax = mesh.aabb.max.max().ceil() + 1.0;
        if (bmax - bmin) != 0.0 {
            bmax += 1.0;
        }

        println!("Range: [{}, {}]", bmin, bmax);

        let mut instance = Self {
            root: Arc::new(Mutex::new(OctreeNode::new(AABB
            {
                min: Vector3::<f32>::new(bmin, bmin, bmin),
                max: Vector3::<f32>::new(bmax, bmax, bmax),
            }))),
            leaves: Vec::new(),
        };

        instance.initialize(Arc::clone(&instance.root));

        instance
    }

    fn initialize(&mut self, mut node: Arc<Mutex<OctreeNode>>) {
        if node.lock().unwrap().aabb.size().min() <= 1.0 {
            self.leaves.push(node);
            return;
        }

        // subdivide
        if node.lock().unwrap().children.is_none() {
            node.lock().unwrap().subdivide();
        }

        match node.lock().unwrap().children {
            None => {}
            Some(ref mut children) => {
                for child in children.iter_mut() {
                    Octree::initialize(self, Arc::clone(child));
                }
            }
        }
    }

    pub fn prune(&mut self) {
        Octree::_prune(Arc::clone(&self.root));
    }

    fn _prune(node: Arc<Mutex<OctreeNode>>) -> bool {
        let mut n = node.lock().unwrap();

        if n.children.is_none() {
            return n.faces.len() == 0;
        }

        match &n.children {
            Some(children) => {
                let mut p = false;
                for child in children {
                    p = p || Octree::_prune(Arc::clone(child));
                }

                if p {
                    n.children = None;
                }

                p
            }
            _ => { true }
        }
    }
}


// match node.children {
//     None => {}
//     Some(ref mut children) => {
//         for child in children.iter_mut() {
//             if triangle_aabb_intersection(&vertices, &child.aabb) {
//                 Octree::insert(self, Box::clone(child), face_index, vertices, max_depth, depth + 1);
//             }
//         }
//     }
// }










