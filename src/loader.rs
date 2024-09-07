use std::fs::File;
use std::io::{self, BufRead};
use nalgebra::Vector3;

use crate::aabb::AABB;

// Define the Mesh structure
#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vector3<f32>>,
    pub indices: Vec<Vector3<u32>>,
    pub aabb: AABB,
}

// Define the Model structure
#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub fn load() -> Result<Model, Box<dyn std::error::Error>> {
    let scale: f32 = 100.0;

    // Open the file
    let file = File::open("assets/stanford-bunny.obj")?;

    // Create a buffered reader
    let reader = io::BufReader::new(file);

    // Vectors to store the raw data
    let mut vertices: Vec<Vector3<f32>> = Vec::new();
    let mut indices: Vec<Vector3<u32>> = Vec::new();
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut min_z = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    let mut max_z = f32::MIN;

    // For the parsed structures
    let mut meshes: Vec<Mesh> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();

        if let Some(first) = parts.next() {
            match first {
                "v" => {
                    let x: f32 = parts.next().unwrap().parse::<f32>().unwrap() * scale;
                    let y: f32 = parts.next().unwrap().parse::<f32>().unwrap() * scale;
                    let z: f32 = parts.next().unwrap().parse::<f32>().unwrap() * scale;
                    vertices.push(<Vector3<f32>>::from([x, y, z]));

                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }

                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }

                    if z < min_z {
                        min_z = z;
                    }
                    if z > max_z {
                        max_z = z;
                    }
                }
                "vn" => {}
                "vt" => {}
                "f" => {
                    let mut vertex_indices = Vec::new();
                    let mut normal_indices = Vec::new();
                    let mut uv_indices = Vec::new();

                    for part in parts {
                        let indices: Vec<_> = part.split('/').collect();
                        vertex_indices.push(indices[0].parse::<u32>().unwrap() - 1);
                        if indices.len() > 1 && !indices[1].is_empty() {
                            uv_indices.push(indices[1].parse::<u32>().unwrap() - 1);
                        }
                        if indices.len() > 2 && !indices[2].is_empty() {
                            normal_indices.push(indices[2].parse::<u32>().unwrap() - 1);
                        }
                    }
                    indices.push(<Vector3<u32>>::from([vertex_indices[0], vertex_indices[1], vertex_indices[2]]));
                }
                _ => {}
            }
        }
    }

    // Create a mesh and add faces to it
    meshes.push(Mesh {
        vertices,
        indices,
        aabb: AABB::new(
            Vector3::new(min_x, min_y, min_z),
            Vector3::new(max_x, max_y, max_z),
        ),
    });

    // Create a model and add meshes to it
    Ok(Model { meshes })
}

