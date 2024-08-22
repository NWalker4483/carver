use kiss3d::nalgebra::{Point3, Vector3};
use stl_io::{IndexedMesh, IndexedTriangle, Triangle, Vector, Vertex};
use crate::errors::CAMError;
use crate::stl_operations::get_bounds;
use crate::tool::{Tool, ToolLibrary};

#[derive(Debug, Clone)]
pub struct Keypoint {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
}

pub trait CAMTask {
    fn process(&mut self, mesh: &IndexedMesh) -> Result<(), CAMError>;
    fn get_keypoints(&self) -> Vec<Keypoint>;
    fn get_tool_id(&self) -> usize;
}

pub struct CAMJOB {
    tasks: Vec<Box<dyn CAMTask>>,
    pub target_mesh: Option<IndexedMesh>,
    pub stock_mesh: Option<IndexedMesh>,
    pub tool_library: ToolLibrary,
}

impl CAMJOB {
    pub fn new() -> Self {
        CAMJOB {
            tasks: Vec::new(),
            target_mesh: None,
            stock_mesh: None,
            tool_library: ToolLibrary::new(),
        }
    }

    pub fn set_mesh(&mut self, mesh: IndexedMesh) -> Result<(), CAMError> {
        self.target_mesh = Some(mesh);
        self.create_stock_mesh()
    }

    pub fn create_stock_mesh(&mut self) -> Result<(), CAMError> {
        if let Some(target_mesh) = &self.target_mesh {
            let stock_mesh = generate_stock_mesh(target_mesh)?;
            self.stock_mesh = Some(stock_mesh);
            Ok(())
        } else {
            Err(CAMError::MeshNotSet)
        }
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
        if let Some(mesh) = &self.target_mesh {
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

    pub fn get_stock_mesh(&self) -> Option<&IndexedMesh> {
        self.stock_mesh.as_ref()
    }

    pub fn get_tasks(&self) -> &Vec<Box<dyn CAMTask>> {
        &self.tasks
    }

    pub fn add_tool(&mut self, tool: Tool) {
        self.tool_library.add_tool(tool);
    }

    pub fn get_tool(&self, id: usize) -> Option<&Tool> {
        self.tool_library.get_tool(id)
    }

    pub fn get_tool_mut(&mut self, id: usize) -> Option<&mut Tool> {
        self.tool_library.get_tool_mut(id)
    }

    pub fn update_to_time_step(&mut self, time_step: usize) {
        // Implement the logic to update the CAM job to a specific time step
        println!("Updating CAM job to time step: {}", time_step);
    }

    pub fn get_tool_position_at_time_step(&self, time_step: usize) -> Option<Point3<f32>> {
        // Implement the logic to get the tool position at a specific time step
        println!("Getting tool position at time step: {}", time_step);
        Some(Point3::new(0.0, 0.0, 0.0)) // Placeholder return value
    }

    pub fn create_simulation_mesh(&self, time_step: usize) -> kiss3d::scene::SceneNode {
        // Implement the logic to create a new simulation mesh
        println!("Creating simulation mesh for time step: {}", time_step);
        // Placeholder: You'll need to actually create and return a SceneNode here
        unimplemented!("create_simulation_mesh not yet implemented")
    }

    pub fn update_simulation_mesh(&self, mesh: &mut kiss3d::scene::SceneNode, time_step: usize) {
        // Implement the logic to update an existing simulation mesh
        println!("Updating simulation mesh for time step: {}", time_step);
    }
}

fn generate_stock_mesh(target_mesh: &IndexedMesh) -> Result<IndexedMesh, CAMError> {
    let (min, max) = get_bounds(target_mesh)?;
    
    // Add some padding to ensure the stock fully encapsulates the target
    let padding = 0.1; // 10% padding
    let min = Point3::new(
        min.x - (max.x - min.x) * padding,
        min.y - (max.y - min.y) * padding,
        min.z - (max.z - min.z) * padding
    );
    let max = Point3::new(
        max.x + (max.x - min.x) * padding,
        max.y + (max.y - min.y) * padding,
        max.z + (max.z - min.z) * padding
    );

    // Define the vertices of the cube
    let vertices: Vec<Vector<f32>> = vec![
        Vector::new([min.x, min.y, min.z]),  // 0
        Vector::new([max.x, min.y, min.z]),  // 1
        Vector::new([max.x, max.y, min.z]),  // 2
        Vector::new([min.x, max.y, min.z]),  // 3
        Vector::new([min.x, min.y, max.z]),  // 4
        Vector::new([max.x, min.y, max.z]),  // 5
        Vector::new([max.x, max.y, max.z]),  // 6
        Vector::new([min.x, max.y, max.z]),  // 7
    ];

    // Define the faces using IndexedTriangle with normals
    let faces: Vec<IndexedTriangle> = vec![
        // Front face (normal: 0, 0, -1)
        IndexedTriangle { normal: Vector::new([0.0, 0.0, -1.0]), vertices: [0, 1, 2] },
        IndexedTriangle { normal: Vector::new([0.0, 0.0, -1.0]), vertices: [0, 2, 3] },
        // Right face (normal: 1, 0, 0)
        IndexedTriangle { normal: Vector::new([1.0, 0.0, 0.0]), vertices: [1, 5, 6] },
        IndexedTriangle { normal: Vector::new([1.0, 0.0, 0.0]), vertices: [1, 6, 2] },
        // Back face (normal: 0, 0, 1)
        IndexedTriangle { normal: Vector::new([0.0, 0.0, 1.0]), vertices: [5, 4, 7] },
        IndexedTriangle { normal: Vector::new([0.0, 0.0, 1.0]), vertices: [5, 7, 6] },
        // Left face (normal: -1, 0, 0)
        IndexedTriangle { normal: Vector::new([-1.0, 0.0, 0.0]), vertices: [4, 0, 3] },
        IndexedTriangle { normal: Vector::new([-1.0, 0.0, 0.0]), vertices: [4, 3, 7] },
        // Top face (normal: 0, 1, 0)
        IndexedTriangle { normal: Vector::new([0.0, 1.0, 0.0]), vertices: [3, 2, 6] },
        IndexedTriangle { normal: Vector::new([0.0, 1.0, 0.0]), vertices: [3, 6, 7] },
        // Bottom face (normal: 0, -1, 0)
        IndexedTriangle { normal: Vector::new([0.0, -1.0, 0.0]), vertices: [4, 5, 1] },
        IndexedTriangle { normal: Vector::new([0.0, -1.0, 0.0]), vertices: [4, 1, 0] },
    ];
    
    Ok(IndexedMesh { vertices, faces })
}