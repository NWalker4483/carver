use stl_io::IndexedMesh;
use kiss3d::nalgebra::{Vector3, Point3};
use kiss3d::window::Window;
use kiss3d::light::Light;
use std::rc::Rc;
use std::{cell::RefCell, path::Path};
use std::env;
use std::fs::File;
use stl_io::{self, IndexedMesh as StlMesh, Vertex};
use crate::cam_job::Keypoint;
use crate::errors::CAMError;
use crate::cam_job::CAMTask;
use ncollide3d::query::{Ray, RayCast};
use ncollide3d::shape::TriMesh;
use nalgebra::RealField;
use ncollide3d::math::Point as NCPoint;


impl From<IndexedMesh> for TriMesh<f32>  {
    fn from(mesh: IndexedMesh) -> Self {
        let vertices: Vec<NCPoint<f32>> = mesh.vertices.iter()
        .map(|v| NCPoint::new(v[0], v[1], v[2]))
        .collect();
    
    let indices: Vec<Point3<usize>> = mesh.faces.iter()
        .map(|f| Point3::new(f.vertices[0] as usize, f.vertices[1] as usize, f.vertices[2] as usize))
        .collect();
    
    TriMesh::new(vertices, indices, None)
    }
}

impl From<IndexedMesh> for kiss3d::resource::Mesh {
    fn from(mesh: IndexedMesh) -> Self {
        let vertices: Vec<Point3<f32>> = mesh.vertices.iter()
        .map(|v| Point3::new(v[0], v[1], v[2]))
        .collect();
    
    let indices: Vec<Point3<u16>> = mesh.faces.iter()
        .map(|f| Point3::new(f.vertices[0] as u16, f.vertices[1] as u16, f.vertices[2] as u16))
        .collect();

    kiss3d::resource::Mesh::new(vertices, indices, None, None, false)

    }
}
