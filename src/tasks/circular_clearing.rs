use crate::prelude::*;
use crate::cam_job::{CAMTask, Keypoint};
use crate::errors::CAMError;
use crate::stl_operations::{indexed_mesh_to_trimesh, is_point_inside_model};
use kiss3d::nalgebra::{Point3, Vector3, Isometry3};
use ncollide3d::query::{Ray, RayCast};
use ncollide3d::shape::TriMesh;
use stl_io::IndexedMesh;

pub struct CircularClearing {
    start_position: Point3<f32>,
    end_position: Point3<f32>,
    num_layers: usize,
    initial_radius: f32,
    num_points_per_ring: usize,
    max_shrink_amount: f32,
    min_shrink_amount: f32,
    keypoints: Vec<Keypoint>,
    layer_completed: Vec<bool>,
}

impl CircularClearing {
    pub fn new(
        start_position: Point3<f32>,
        end_position: Point3<f32>,
        num_layers: usize,
        initial_radius: f32,
        num_points_per_ring: usize,
        max_shrink_amount: f32,
        min_shrink_amount: f32,
    ) -> Self {
        CircularClearing {
            start_position,
            end_position,
            num_layers,
            initial_radius,
            num_points_per_ring,
            max_shrink_amount,
            min_shrink_amount,
            keypoints: Vec::new(),
            layer_completed: vec![false; num_layers],
        }
    }

    fn generate_ring_points(&self, center: &Point3<f32>, radius: f32, normal: &Vector3<f32>) -> Vec<(Point3<f32>, Vector3<f32>)> {
        let mut points = Vec::new();
        
        let v1 = if normal.x.abs() < normal.y.abs() && normal.x.abs() < normal.z.abs() {
            Vector3::new(1.0, 0.0, 0.0).cross(normal).normalize()
        } else {
            Vector3::new(0.0, 1.0, 0.0).cross(normal).normalize()
        };
        let v2 = normal.cross(&v1);

        for i in 0..self.num_points_per_ring {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / self.num_points_per_ring as f32;
            let direction = (v1 * angle.cos() + v2 * angle.sin()).normalize();
            let point = center + direction * radius;
            points.push((point, direction));
        }

        points
    }

    fn is_ring_valid(&self, center: &Point3<f32>, radius: f32, normal: &Vector3<f32>, tri_mesh: &TriMesh<f32>) -> bool {
        let points = self.generate_ring_points(&center, radius, &normal);
        let num_points = points.len();
        if (radius < 0.001){
            return false;
        }
    
        for i in 0..num_points {
            let (current_point, _) = points[i];
            let (next_point, _) = points[(i + 1) % num_points];
    
            let direction = next_point - current_point;
            let ray = Ray::new(ncollide3d::math::Point::from(current_point.coords), direction);
    
            if let Some(toi) = tri_mesh.toi_with_ray(&Isometry3::identity(), &ray, std::f32::MAX, false) {
                // If the intersection point is before the next point, the ring intersects with the model
                if toi < direction.norm() || toi < 10. {
                    return false;
                }
            }
        }
    
        true
    }
    

    fn find_max_valid_shrink(&self, center: &Point3<f32>, current_radius: f32, normal: &Vector3<f32>, tri_mesh: &TriMesh<f32>) -> Option<f32> {
        if self.is_ring_valid(center, current_radius - self.max_shrink_amount, normal, tri_mesh) {
            return Some(self.max_shrink_amount);
        }

        if !self.is_ring_valid(center, current_radius - self.min_shrink_amount, normal, tri_mesh) {
            return None;
        }

        let mut low = self.min_shrink_amount;
        let mut high = self.max_shrink_amount;

        while high - low > 0.001 {  // Precision threshold
            let mid = (low + high) / 2.0;
            if self.is_ring_valid(center, current_radius - mid, normal, tri_mesh) {
                low = mid;
            } else {
                high = mid;
            }
        }

        Some(low)
    }

    fn process_phase(&mut self, tri_mesh: &TriMesh<f32>, layer_positions: &[Point3<f32>], current_radii: &mut [f32], normal: &Vector3<f32>) -> bool {
        let mut any_valid_ring = false;

        for layer in 0..self.num_layers {
            if self.layer_completed[layer] {
                continue;  // Skip already completed layers
            }

            let center = &layer_positions[layer];
            let radius = &mut current_radii[layer];

            let proposed_shrink_amount = self.find_max_valid_shrink(center, *radius, normal, tri_mesh);
            println!("Layer {}: Center {:?}, Current radius {}, Proposed shrink amount {:?}", layer, center, radius, proposed_shrink_amount);
            
            if let Some(shrink_amount) = proposed_shrink_amount {
                let new_radius = (*radius - shrink_amount);//.max(self.min_shrink_amount);
                println!("Layer {}: Shrinking from {} to {}", layer, *radius, new_radius);
                
                let ring_points = self.generate_ring_points(center, new_radius, normal);
                for (point, direction) in ring_points {
                    self.keypoints.push(Keypoint {
                        position: point,
                        normal: direction,
                    });
                }
                
                *radius = new_radius;
                any_valid_ring = true;
            } else {
                self.layer_completed[layer] = true;
                println!("Layer {} completed: No valid shrink amount found", layer);
            }
        }

        any_valid_ring
    }
}

impl CAMTask for CircularClearing {
    fn get_tool_id(&self) -> usize {
        1 as usize
    }
    fn process(&mut self, mesh: &IndexedMesh) -> Result<(), CAMError> {
        println!("Processing circular clearing from {:?} to {:?}", self.start_position, self.end_position);
        let tri_mesh = indexed_mesh_to_trimesh(mesh);

        self.keypoints.clear();
        self.layer_completed = vec![false; self.num_layers];

        let layer_height = (self.end_position - self.start_position).norm() / (self.num_layers - 1) as f32;
        let normal = (self.end_position - self.start_position).normalize();
        let layer_positions: Vec<Point3<f32>> = (0..self.num_layers)
            .map(|layer| self.start_position + normal * (layer as f32 * layer_height))
            .collect();

        let mut current_radii = vec![self.initial_radius; self.num_layers];

        let mut phase = 0;
        loop {
            let any_valid_ring = self.process_phase(&tri_mesh, &layer_positions, &mut current_radii, &normal);
            
            println!("Completed phase {}", phase);
            phase += 1;

            if !any_valid_ring && self.layer_completed.iter().all(|&completed| completed==true) {
                println!("All layers completed or no valid rings found");
                break;
            }
        }

        println!("Generated {} keypoints for circular clearing", self.keypoints.len());
        Ok(())
    }

    fn get_keypoints(&self) -> Vec<Keypoint> {
        self.keypoints.clone()
    }
}