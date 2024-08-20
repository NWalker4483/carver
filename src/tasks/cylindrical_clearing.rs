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
    num_rays: usize,
    keypoints: Vec<Keypoint>,
    position: Point3<f32>,
    normal: Vector3<f32>,
    bounding_radius: f32,
}

impl ContourTrace {
    pub fn new(num_rays: usize, position: Point3<f32>, normal: Vector3<f32>, mesh: &IndexedMesh) -> Self {
        let (min_bound, max_bound) = get_bounds(mesh).unwrap();
        let center = (min_bound + max_bound.coords) * 0.5;
        let bounding_radius = (max_bound - min_bound).norm() * 0.5;

        ContourTrace {
            num_rays,
            keypoints: Vec::new(),
            position,
            normal: normal.normalize(),
            bounding_radius,
        }
    }

    fn cast_ray(&self, tri_mesh: &TriMesh<f32>, origin: Point3<f32>, direction: Vector3<f32>) -> Option<Keypoint> {
        let ray = Ray::new(NCPoint::from(origin.coords), direction);
        let intersection = tri_mesh.toi_and_normal_with_ray(&Isometry3::identity(), &ray, 100 as f32, true);

        intersection.map(|intersection| {
            let point = origin + direction * intersection.toi;
            Keypoint {
                position: point,
                normal: intersection.normal,
            }
        })
    }
}

impl CAMTask for ContourTrace {
    fn process(&mut self, mesh: &IndexedMesh) -> Result<(), CAMError> {
        println!("Processing contour trace at position: {:?}, normal: {:?}", self.position, self.normal);
        let tri_mesh = indexed_mesh_to_trimesh(mesh);

        self.keypoints.clear();

        // Calculate two perpendicular vectors in the plane
        let v1 = if self.normal.x.abs() < self.normal.y.abs() && self.normal.x.abs() < self.normal.z.abs() {
            Vector3::new(1.0, 0.0, 0.0).cross(&self.normal).normalize()
        } else {
            Vector3::new(0.0, 1.0, 0.0).cross(&self.normal).normalize()
        };
        let v2 = self.normal.cross(&v1);

        for i in 0..self.num_rays {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / self.num_rays as f32;
            let direction = -(v1 * angle.cos() + v2 * angle.sin()).normalize();
            
            // Calculate the origin point outside the bounding sphere
            let origin = self.position + (v1 * angle.cos() + v2 * angle.sin()) * (self.bounding_radius + 1.0);

            if let Some(keypoint) = self.cast_ray(&tri_mesh, origin, direction) {
                // Check if the keypoint is close to the plane defined by position and normal
                let distance_to_plane = (keypoint.position - self.position).dot(&self.normal).abs();
                if distance_to_plane < 0.1 {
                    self.keypoints.push(keypoint);
                }
            }
        }

        println!("Generated {} keypoints for contour trace", self.keypoints.len());
        Ok(())
    }

    fn get_keypoints(&self) -> Vec<Keypoint> {
        self.keypoints.clone()
    }
}