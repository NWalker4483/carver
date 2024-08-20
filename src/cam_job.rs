use kiss3d::nalgebra::{Point3, Vector3};
use stl_io::IndexedMesh;
use std::fmt;
use std::error::Error;

use crate::errors::CAMError;
use crate::stl_operations::get_bounds;
use crate::tasks::multicontourtrace::MultiContourTrace;

#[derive(Debug, Clone)]
pub struct Keypoint {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
}

pub trait CAMTask {
    fn process(&mut self, mesh: &IndexedMesh) -> Result<(), CAMError>;
    fn get_keypoints(&self) -> Vec<Keypoint>;
}

pub struct CAMJOB {
    tasks: Vec<Box<dyn CAMTask>>,
    mesh: Option<IndexedMesh>,
}

impl CAMJOB {
    pub fn new() -> Self {
        CAMJOB {
            tasks: Vec::new(),
            mesh: None,
        }
    }

    pub fn set_mesh(&mut self, mesh: IndexedMesh) {
        self.mesh = Some(mesh);
    }

    pub fn add_task(&mut self, task: Box<dyn CAMTask>) {
        self.tasks.push(task);
    }

    pub fn get_next_task(&self) -> Option<&dyn CAMTask> {
        self.tasks.first().map(AsRef::as_ref)
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }

    pub fn build(&mut self) -> Result<(), CAMError> {
        if let Some(mesh) = &self.mesh {
            for task in &mut self.tasks {
                task.process(mesh)?;
            }
            Ok(())
        } else {
            Err(CAMError::MeshNotSet)
        }
    }

    pub fn gather_keypoints(&self) -> Vec<Keypoint> {
        self.tasks.iter().flat_map(|task| task.get_keypoints()).collect()
    }
}
