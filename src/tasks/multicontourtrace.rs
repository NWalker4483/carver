
use kiss3d::nalgebra::{Point3, Vector3};
use stl_io::IndexedMesh;
use crate::cam_job::{CAMTask, Keypoint};
use crate::errors::CAMError;
use crate::stl_operations::get_bounds;
use super::ContourTrace;

pub struct MultiContourTrace {
    start_height: f32,
    end_height: f32,
    num_layers: usize,
    num_rays: usize,
    ray_length: f32,
    keypoints: Vec<Keypoint>,
}

impl MultiContourTrace {
    pub fn new(
        start_height: f32,
        end_height: f32,
        num_layers: usize,
        num_rays: usize,
        ray_length: f32,
    ) -> MultiContourTrace {
        MultiContourTrace {
            start_height,
            end_height,
            num_layers,
            num_rays,
            ray_length,
            keypoints: Vec::new(),
        }
    }
}

impl CAMTask for MultiContourTrace {
    fn process(&mut self, mesh: &IndexedMesh) -> Result<(), CAMError> {
        println!("Processing multi-contour trace from {} to {} with {} layers",
                 self.start_height, self.end_height, self.num_layers);

        let (min_bound, max_bound) = get_bounds(mesh).map_err(|e| CAMError::ProcessingError(e.to_string()))?;
        let height_step = (self.end_height - self.start_height) / self.num_layers as f32;

        self.keypoints.clear();

        for i in 0..=self.num_layers {
            let layer_height = self.start_height + i as f32 * height_step;
            let mut contour_trace = ContourTrace::new(self.num_rays, self.ray_length, layer_height);
            
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