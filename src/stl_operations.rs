use crate::prelude::*;
use std::path::Path;
use std::fs::File;
use anyhow::Result;
use stl_io::{self, IndexedMesh, Vertex};
use kiss3d::nalgebra::Point3;
use crate::errors::CAMError;
use ncollide3d::query::{Ray, RayCast};
use ncollide3d::shape::TriMesh;
use ncollide3d::math::Point as NCPoint;
use kiss3d::nalgebra::{ Vector3, Isometry3};



pub fn load_stl(filename: &Path) -> Result<IndexedMesh> {
    let mut file = File::open(filename)?;
    Ok(stl_io::read_stl(&mut file)?)
}
   /// Converts IndexedMesh to ncollide3d::shape::TriMesh
pub fn indexed_mesh_to_trimesh(mesh: &IndexedMesh) -> TriMesh<f32> {
    let vertices: Vec<NCPoint<f32>> = mesh.vertices.iter()
        .map(|v| NCPoint::new(v[0], v[1], v[2]))
        .collect();
    
    let indices: Vec<Point3<usize>> = mesh.faces.iter()
        .map(|f| Point3::new(f.vertices[0] as usize, f.vertices[1] as usize, f.vertices[2] as usize))
        .collect();

    TriMesh::new(vertices, indices, None)
}

    /// Checks if a point is inside the 3D model.
pub fn is_point_inside_model( point: &Point3<f32>, normal: &Vector3<f32>, tri_mesh: &TriMesh<f32>) -> bool {
        let epsilon = 1e-6;
        let ray_start = point + normal * epsilon;
        let ray = Ray::new(ncollide3d::math::Point::from(ray_start.coords), *normal);

        let forward_hit = tri_mesh.toi_and_normal_with_ray(&Isometry3::identity(), &ray, std::f32::MAX, true);
        let backward_ray = Ray::new(ncollide3d::math::Point::from(ray_start.coords), -normal);
        let backward_hit = tri_mesh.toi_and_normal_with_ray(&Isometry3::identity(), &backward_ray, std::f32::MAX, true);

        forward_hit.is_some() != backward_hit.is_some()
    }

pub fn center_and_scale_mesh(mesh: &mut IndexedMesh) -> (f32, f32) {
    let (min, max) = get_bounds(mesh).expect("Failed to get mesh bounds");
    let center = [
        (min.x + max.x) / 2.0,
        (min.y + max.y) / 2.0,
        0.0, // We don't center vertically
    ];
    let size = [
        max.x - min.x,
        max.y - min.y,
        max.z - min.z,
    ];
    let scale = 1.0;// / size.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    let min_z = min.z * scale;
    let max_z = max.z * scale;

    for vertex in &mut mesh.vertices {
        let scaled_vertex = [
            (vertex[0] - center[0]) * scale,
            (vertex[1] - center[1]) * scale,
            (vertex[2] - min.z) * scale + min_z, // Adjust z to start from min_z
        ];
        *vertex = Vertex::new(scaled_vertex);
    }

    (min_z, max_z)
}

pub fn get_bounds(mesh: &IndexedMesh) -> Result<(Point3<f32>, Point3<f32>), CAMError> {
    mesh.vertices.iter()
        .try_fold((Point3::new(f32::MAX, f32::MAX, f32::MAX), Point3::new(f32::MIN, f32::MIN, f32::MIN)), 
                  |(min, max), v| {
            let new_min = Point3::new(min.x.min(v[0]), min.y.min(v[1]), min.z.min(v[2]));
            let new_max = Point3::new(max.x.max(v[0]), max.y.max(v[1]), max.z.max(v[2]));
            if new_min.coords.iter().all(|&x| x.is_finite()) && new_max.coords.iter().all(|&x| x.is_finite()) {
                Ok((new_min, new_max))
            } else {
                Err(CAMError::InvalidMesh("Mesh contains invalid vertex values".into()))
            }
        })
}

pub fn mesh_to_kiss3d(mesh: &IndexedMesh) -> kiss3d::resource::Mesh {
    let vertices: Vec<Point3<f32>> = mesh.vertices.iter()
        .map(|v| Point3::new(v[0], v[1], v[2]))
        .collect();
    
    let indices: Vec<Point3<u16>> = mesh.faces.iter()
        .map(|f| Point3::new(f.vertices[0] as u16, f.vertices[1] as u16, f.vertices[2] as u16))
        .collect();

    kiss3d::resource::Mesh::new(vertices, indices, None, None, false)
}