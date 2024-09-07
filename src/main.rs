use std::sync::{Arc, Mutex};
use raylib::camera::Camera3D;
use raylib::drawing::RaylibDraw;
use raylib::init;
use raylib::prelude::{CameraMode, RaylibDraw3D, RaylibDrawHandle, RaylibMode3D, RaylibMode3DExt};
use crate::octree::OctreeNode;

use rayon::prelude::*;

mod loader;
mod octree;
mod vec_ops;
mod aabb;

fn draw_octree(d: &mut RaylibMode3D<RaylibDrawHandle>, node: &OctreeNode) {
    // draw only if leaf node
    if !node.faces.is_empty() {
        let size = node.aabb.size();
        d.draw_cube(
            raylib::prelude::Vector3::new(node.aabb.min.x, node.aabb.min.y, node.aabb.min.z),
            size.x, size.y, size.z,
            raylib::prelude::Color::new(0, 0, 255, 255),
        );
    }

    match node.children {
        Some(ref children) => {
            for child in children.iter() {
                draw_octree(d, &child.lock().unwrap());
            }
        }
        None => {}
    }
}

fn main() {
    let model = loader::load().unwrap();
    let mesh = &model.meshes[0];
    let mut root = Arc::new(Mutex::new(OctreeNode::new(mesh.aabb)));

    mesh.indices.par_iter().for_each(|index| {
        root.lock().unwrap().insert(
            0,
            &[
                mesh.vertices[index[0] as usize],
                mesh.vertices[index[1] as usize],
                mesh.vertices[index[2] as usize],
            ],
            4,
            0,
        )
    });

    let WINDOW_WIDTH = 640;
    let WINDOW_HEIGHT = 480;
    let (mut rl, thread) = init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Hello, world!")
        .build();

    let mut camera = Camera3D::perspective(
        raylib::prelude::Vector3::new(4.0, 2.0, 4.0),
        raylib::prelude::Vector3::new(0.0, 1.8, 0.0),
        raylib::prelude::Vector3::new(0.0, 1.0, 0.0),
        60.0,
    );

    rl.set_target_fps(60);
    camera.position = raylib::prelude::Vector3::new(40.0, 2.0, 4.0);

    while !rl.window_should_close() {
        rl.update_camera(&mut camera, CameraMode::CAMERA_ORBITAL);

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(raylib::prelude::Color::DARKGREEN);
        {
            let mut d2 = d.begin_mode3D(camera);

            d2.draw_plane(
                raylib::prelude::Vector3::new(0.0, 0.0, 0.0),
                raylib::prelude::Vector2::new(32.0, 32.0),
                raylib::prelude::Color::LIGHTGRAY,
            );
            draw_octree(&mut d2, &root.lock().unwrap());
        }
    }
}