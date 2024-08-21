use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum CAMError {
    InvalidMesh(String),
    MeshNotSet,
    ProcessingError(String),
}

impl fmt::Display for CAMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CAMError::MeshNotSet => write!(f, "Mesh not set for CAM job"),
            CAMError::InvalidMesh(msg) => write!(f, "Invalid mesh: {}", msg),
            CAMError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}
