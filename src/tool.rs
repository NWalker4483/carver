use std::cell::RefCell;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use kiss3d::nalgebra::{Point3, Vector3};

pub struct Tool {
    pub id: usize,
    pub name: String,
    pub model: RefCell<SceneNode>,
    pub length: f32,
    pub diameter: f32,
}

impl Tool {
    pub fn new(id: usize, name: String, window: &mut Window, length: f32, diameter: f32) -> Self {
        let mut model = window.add_cylinder(diameter / 2.0, length);
        model.set_color(0.8, 0.8, 0.8); // Light gray color
        model.set_visible(false);

        Tool {
            id,
            name,
            model: RefCell::new(model),
            length,
            diameter,
        }
    }

    pub fn set_position(&self, position: Point3<f32>) {
        self.model.borrow_mut().set_local_translation(kiss3d::nalgebra::Translation3::from(position.coords));
    }

    pub fn set_orientation(&self, direction: Vector3<f32>) {
        let rotation = kiss3d::nalgebra::UnitQuaternion::rotation_between(
            &Vector3::new(0.0, 0.0, 1.0),
            &direction.normalize(),
        )
        .unwrap_or_else(kiss3d::nalgebra::UnitQuaternion::identity);
        self.model.borrow_mut().set_local_rotation(rotation);
    }

    pub fn set_visible(&self, visible: bool) {
        self.model.borrow_mut().set_visible(visible);
    }
}

pub struct ToolLibrary {
    tools: Vec<Tool>,
}

impl ToolLibrary {
    pub fn new() -> Self {
        ToolLibrary { tools: Vec::new() }
    }

    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    pub fn get_tool(&self, id: usize) -> Option<&Tool> {
        self.tools.iter().find(|&tool| tool.id == id)
    }

    pub fn get_tool_mut(&mut self, id: usize) -> Option<&mut Tool> {
        self.tools.iter_mut().find(|tool| tool.id == id)
    }
}