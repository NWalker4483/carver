use kiss3d::nalgebra::{Point3, Vector3, Unit, Isometry3};
use stl_io::IndexedMesh;
use crate::cam_job::Keypoint;
use crate::errors::CAMError;
use crate::stl_operations::{get_bounds, indexed_mesh_to_trimesh};
use crate::cam_job::CAMTask;
use ncollide3d::query::{Ray, RayCast};
use ncollide3d::shape::TriMesh;
use ncollide3d::math::Point as NCPoint;

pub struct ContourTrace {
    ray_length: f32,
    num_rays: usize,
    keypoints: Vec<Keypoint>,
    layer_height: f32,
}

impl ContourTrace {
    pub fn new(num_rays: usize, ray_length: f32, layer_height: f32) -> Self {
        ContourTrace {
            num_rays,
            ray_length,
            keypoints: Vec::new(),
            layer_height,
        }
    }

    fn cast_ray(&self, tri_mesh: &TriMesh<f32>, origin: Point3<f32>, direction: Vector3<f32>) -> Option<Keypoint> {
        let ray = Ray::new(NCPoint::from(origin.coords), direction);
        let intersection = tri_mesh.toi_and_normal_with_ray(&Isometry3::identity(), &ray, self.ray_length, true);

        intersection.map(|intersection| {
            let point = origin + direction * intersection.toi;
            Keypoint {
                position: point,
                normal: intersection.normal, // Use the normal from the intersection
            }
        })
    }

    fn calculate_model_center(&self, min_bound: &Point3<f32>, max_bound: &Point3<f32>) -> Point3<f32> {
        (min_bound + max_bound.coords) * 0.5
    }
}

impl CAMTask for ContourTrace {
    fn process(&mut self, mesh: &IndexedMesh) -> Result<(), CAMError> {
        println!("Processing contour trace for layer height: {}", self.layer_height);
        let (min_bound, max_bound) = get_bounds(mesh).map_err(|e| CAMError::ProcessingError(e.to_string()))?;
        let tri_mesh = indexed_mesh_to_trimesh(mesh);
        
        let model_center = self.calculate_model_center(&min_bound, &max_bound);
        let max_radius = ((max_bound.x - min_bound.x).powi(2) + (max_bound.y - min_bound.y).powi(2)).sqrt() / 2.0;
        
        self.keypoints.clear();

        for i in 0..self.num_rays {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / self.num_rays as f32;
            
            // Calculate the origin point at the current layer height and on the circumference
            let origin = Point3::new(
                model_center.x + angle.cos() * max_radius,
                model_center.y + angle.sin() * max_radius,
                self.layer_height
            );

            // Calculate the direction towards the Z-axis center
            let direction = Vector3::new(
                model_center.x - origin.x,
                model_center.y - origin.y,
                0.0 // We want to keep it in the XY plane
            ).normalize();

            if let Some(keypoint) = self.cast_ray(&tri_mesh, origin, direction) {
                if (keypoint.position.z - self.layer_height).abs() < 0.001 { // Allow for small floating-point errors
                    self.keypoints.push(keypoint);
                }
            }
        }

        println!("Generated {} keypoints for layer height {}", self.keypoints.len(), self.layer_height);
        Ok(())
    }

    fn get_keypoints(&self) -> Vec<Keypoint> {
        self.keypoints.clone()
    }
}