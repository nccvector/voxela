#![allow(non_snake_case)]

use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{ThreadId};
use std::time::Duration;
use raylib::camera::Camera3D;
use raylib::drawing::RaylibDraw;
use raylib::init;
use raylib::prelude::{CameraMode, RaylibDraw3D, RaylibDrawHandle, RaylibMode3D, RaylibMode3DExt};
use crate::octree::{Octree, OctreeNode};

use rayon::prelude::*;
use crate::aabb::AABB;

mod loader;
mod octree;
mod vec_ops;
mod aabb;

fn draw_octree(d: &mut RaylibMode3D<RaylibDrawHandle>, node: Arc<Mutex<OctreeNode>>) {
    let l = node.lock().unwrap();
    let size = l.aabb.size();

    // draw only if leaf node
    if !l.faces.is_empty() {
        d.draw_cube(
            raylib::prelude::Vector3::new(l.aabb.min.x, l.aabb.min.y, l.aabb.min.z),
            size.x, size.y, size.z,
            raylib::prelude::Color::new(0, 0, 255, 255),
        );
    } else {
        let epsilon = 0.001;
        d.draw_cube_wires(
            raylib::prelude::Vector3::new(l.aabb.min.x, l.aabb.min.y, l.aabb.min.z),
            size.x + epsilon, size.y + epsilon, size.z + epsilon,
            raylib::prelude::Color::new(0, 0, 0, 255),
        );
    }

    match l.children.clone() {
        Some(children) => {
            for child in children {
                draw_octree(d, child);
            }
        }
        _ => {}
    }
}

struct ThreadDump {
    id: Option<ThreadId>,
    totalLeavesToProcess: usize,
    progress: f32,
    done: bool,
}

fn main() {
    let model = loader::load().unwrap();
    let mesh = Arc::new(model.meshes[0].clone());

    // Construct a vec of aabbs for each face
    let mut aabbList = vec![];
    for index in mesh.indices.clone().iter() {
        aabbList.push(
            AABB::from(&[
                mesh.vertices[index[0] as usize],
                mesh.vertices[index[1] as usize],
                mesh.vertices[index[2] as usize],
            ])
        );
    }
    let aabbListPtr: Arc<Vec<AABB>> = Arc::new(aabbList);

    let mut octree = Octree::new(&mesh);
    println!("Total leaves: {}", octree.leaves.len());

    let mut handles = vec![];
    let nthreads = 16;

    // Initialize thread dumps
    let mut threadDumps = vec![];
    for i in 0..nthreads {
        threadDumps.push(Arc::new(Mutex::new(ThreadDump {
            id: None,
            totalLeavesToProcess: 0,
            progress: 0.0,
            done: false,
        })));
    }

    // Prepare data for each thread and start
    let leafchunksize = octree.leaves.len() / nthreads;
    println!("Leaves per thread: {}", leafchunksize);
    for i in 0..nthreads {
        let mut leaves = vec![];
        for c in 0..leafchunksize {
            let leafIndex = i * leafchunksize + c;
            let leafPtr = Arc::clone(&octree.leaves[leafIndex]);
            leaves.push(leafPtr);
        }
        let mesh = Arc::clone(&mesh);
        let aabbListPtr = Arc::clone(&aabbListPtr);

        let threadDump = Arc::clone(&threadDumps[i]);

        let handle = thread::spawn(move || {
            // Thread context
            threadDump.lock().unwrap().id = Some(thread::current().id());

            let totalLeavesForThread = leaves.len();
            threadDump.lock().unwrap().totalLeavesToProcess = totalLeavesForThread;
            println!("Leaf size for {:?} thread: {}", thread::current().id(), totalLeavesForThread);

            for t in 0..leaves.iter().len() {
                // println!("{:?}", thread::current().id());
                for (faceIndex, index) in mesh.indices.iter().enumerate() {
                    leaves[t].lock().unwrap().insert_face(
                        faceIndex,
                        &[
                            mesh.vertices[index[0] as usize],
                            mesh.vertices[index[1] as usize],
                            mesh.vertices[index[2] as usize],
                        ],
                        &aabbListPtr[faceIndex],
                    )
                }

                threadDump.lock().unwrap().progress = t as f32 / totalLeavesForThread as f32 * 100.0;
                // println!("{:?}", threadDump.lock().unwrap().progress);
            }

            // Inform other threads about done status
            threadDump.lock().unwrap().done = true;
        });

        handles.push(handle);
    }

    // Show progress thread
    handles.push(thread::spawn(move || {
        loop {
            let mut allDone = true;
            for threadDump in &threadDumps {
                let d = threadDump.lock().unwrap();
                println!("Thread id: {:?}\tLeaves: {}\tProgress: {}", d.id, d.totalLeavesToProcess, d.progress);

                allDone = allDone & d.done;
            }

            if allDone {
                break;
            }

            thread::sleep(Duration::from_secs(2));
        }
    }));

    // Join all threads
    for handle in handles {
        handle.join();
    }

    // Delete nodes leading to empty leaves
    octree.prune();

    // RAY LIB DEBUG VIEWER
    let WINDOW_WIDTH = 1280;
    let WINDOW_HEIGHT = 720;
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
    camera.position = raylib::prelude::Vector3::new(40.0, 4.0, 20.0);

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
            draw_octree(&mut d2, Arc::clone(&octree.root));
        }
    }
}