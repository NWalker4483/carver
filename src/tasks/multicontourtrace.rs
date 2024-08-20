
use kiss3d::nalgebra::{Point3, Vector3};
use stl_io::IndexedMesh;
use crate::cam_job::{CAMTask, Keypoint};
use crate::errors::CAMError;
use crate::stl_operations::get_bounds;
use super::ContourTrace;

pub struct MultiContourTrace {
    start_position: Point3<f32>,
    end_position: Point3<f32>,
    num_layers: usize,
    num_rays: usize,
    keypoints: Vec<Keypoint>,
}

impl MultiContourTrace {
    pub fn new(
        start_position: Point3<f32>,
        end_position: Point3<f32>,
        num_layers: usize,
        num_rays: usize,
    ) -> MultiContourTrace {
        MultiContourTrace {
            start_position,
            end_position,
            num_layers,
            num_rays,
            keypoints: Vec::new(),
        }
    }
}

impl CAMTask for MultiContourTrace {
    fn process(&mut self, mesh: &IndexedMesh) -> Result<(), CAMError> {
        println!("Processing multi-contour trace from {:?} to {:?} with {} layers",
                 self.start_position, self.end_position, self.num_layers);

        self.keypoints.clear();

        let direction = self.end_position - self.start_position;
        let normal = direction.normalize();

        for i in 0..=self.num_layers {
            let t = i as f32 / self.num_layers as f32;
            let position = self.start_position + direction * t;

            let mut contour_trace = ContourTrace::new(self.num_rays, position, normal, mesh);

            contour_trace.process(mesh)?;
            self.keypoints.extend(contour_trace.get_keypoints());
        }

        println!("Generated {} total keypoints across all layers", self.keypoints.len());
        Ok(())
    }

    fn get_keypoints(&self) -> Vec<Keypoint> {
        self.keypoints.clone()
    }
}