use std::sync::Arc;
use std::sync::Mutex;
use kiss3d::window::Window;
use kiss3d::scene::SceneNode;
use kiss3d::nalgebra::{Point3, Vector3, Translation3, UnitQuaternion, Isometry3};
use kiss3d::conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget, UiCell};
use kiss3d::conrod::widget_ids;
use stl_io::IndexedMesh;
use crate::cam_job::{CAMJOB, Keypoint};
use crate::tool::Tool;

widget_ids! {
    pub struct Ids {
        process_button,
        play_pause_button,
        toggle_mesh_button,
        toggle_stock_mesh_button,
        toggle_keypoints_button,
        toggle_keypoint_lines_button,
        layers_text,
        current_layer_text,
        rays_text,
        ray_length_text,
        animation_speed_text,
        origin_x_text,
        origin_y_text,
        origin_z_text,
        origin_x_slider,
        origin_y_slider,
        origin_z_slider,
        time_step_text,
        time_step_slider,
        toggle_simulation_mesh_button,
    }
}

pub struct AppState {
    pub mesh: IndexedMesh,
    pub cam_job: Arc<Mutex<CAMJOB>>,
    pub num_layers: usize,
    pub num_rays: usize,
    pub ray_length: f32,
    pub is_playing: bool,
    pub current_layer: usize,
    pub animation_speed: f32,
    pub show_mesh: bool,
    pub show_stock_mesh: bool,
    pub show_keypoints: bool,
    pub show_keypoint_lines: bool,
    pub current_keypoint: usize,
    pub job_origin: Isometry3<f32>,
    pub keypoint_spheres: Vec<SceneNode>,
    pub stock_mesh: SceneNode,
    pub current_time_step: usize,
    pub max_time_steps: usize,
    pub show_simulation_mesh: bool,
    pub simulation_mesh: Option<SceneNode>,
    ids: Ids,
}
impl AppState {
    pub fn new(mesh: IndexedMesh, cam_job: CAMJOB, stock_mesh: SceneNode, ui: &mut UiCell) -> Self {
        AppState {
            mesh: mesh.clone(),
            cam_job: Arc::new(Mutex::new(cam_job)),
            num_layers: 40,
            num_rays: 100,
            ray_length: 0.9,
            is_playing: false,
            current_layer: 0,
            animation_speed: 1.0,
            show_mesh: true,
            show_stock_mesh: true,
            show_keypoints: true,
            show_keypoint_lines: true,
            current_keypoint: 0,
            job_origin: Isometry3::identity(),
            keypoint_spheres: Vec::new(),
            stock_mesh,
            current_time_step: 0,
            max_time_steps: 100,
            show_simulation_mesh: false,
            simulation_mesh: None,
            ids: Ids::new(ui.widget_id_generator()),
        }
    }

    pub fn animate(&mut self) {
        let keypoints = self.cam_job.lock().unwrap().gather_keypoints();
        if !keypoints.is_empty() {
            self.current_keypoint = (self.current_keypoint + 1) % keypoints.len();
            let keypoint = &keypoints[self.current_keypoint];
            let transformed_position = self.job_origin * keypoint.position;
            
            let mut cam_job = self.cam_job.lock().unwrap();
            let task = cam_job.get_tasks().get(0).unwrap();
            let tool_id = task.get_tool_id();
            if let Some(tool) = cam_job.get_tool_mut(tool_id) {
                tool.set_position(transformed_position);
                tool.set_orientation(keypoint.normal);
                tool.set_visible(true);
            }
        }
    }

    pub fn draw_keypoint_lines(&self, window: &mut Window) {
        if !self.show_keypoint_lines {
            return;
        }
    
        let cam_job = self.cam_job.lock().unwrap();
        let tasks = cam_job.get_tasks();
        for (task_index, task) in tasks.iter().enumerate() {
            let keypoints = task.get_keypoints();
            let color = get_task_color(task_index);
            for keypoint in keypoints {
                let start = self.job_origin * keypoint.position;
                let end = start + self.job_origin.rotation * (keypoint.normal * self.ray_length);
                window.draw_line(&start, &end, &Point3::from(color));
            }
        }
    }

    pub fn update_simulation(&mut self) {
        println!("Updating simulation for time step: {}", self.current_time_step);
        let mut cam_job = self.cam_job.lock().unwrap();
        cam_job.update_to_time_step(self.current_time_step);
    }

    pub fn generate_simulation_mesh(&mut self) {
        println!("Generating simulation mesh for time step: {}", self.current_time_step);
        let cam_job = self.cam_job.lock().unwrap();
        if let Some(sim_mesh) = &mut self.simulation_mesh {
            cam_job.update_simulation_mesh(sim_mesh, self.current_time_step);
        } else {
            let new_mesh = cam_job.create_simulation_mesh(self.current_time_step);
            self.simulation_mesh = Some(new_mesh);
        }
    }

    pub fn update_tool_position(&mut self) {
        let mut cam_job = self.cam_job.lock().unwrap();
        if let Some(tool_position) = cam_job.get_tool_position_at_time_step(self.current_time_step) {
            let transformed_position = self.job_origin * tool_position;
            let task = cam_job.get_tasks().get(0).unwrap();
            let tool_id = task.get_tool_id();
            if let Some(tool) = cam_job.get_tool_mut(tool_id) {
                tool.set_position(transformed_position);
                // You might also want to update the tool orientation here
            }
        }
    }

    pub fn toggle_mesh_visibility(&mut self) {
        self.show_mesh = !self.show_mesh;
        // Implement the logic to show/hide the mesh in your rendering engine
    }

    pub fn toggle_stock_mesh_visibility(&mut self) {
        self.show_stock_mesh = !self.show_stock_mesh;
        self.stock_mesh.set_visible(self.show_stock_mesh);
    }

    pub fn toggle_keypoints_visibility(&mut self) {
        self.show_keypoints = !self.show_keypoints;
        for sphere in &mut self.keypoint_spheres {
            sphere.set_visible(self.show_keypoints);
        }
    }

    pub fn toggle_keypoint_lines_visibility(&mut self) {
        self.show_keypoint_lines = !self.show_keypoint_lines;
    }

    pub fn toggle_simulation_mesh_visibility(&mut self) {
        self.show_simulation_mesh = !self.show_simulation_mesh;
        if self.show_simulation_mesh {
            self.generate_simulation_mesh();
        }
        if let Some(sim_mesh) = &mut self.simulation_mesh {
            sim_mesh.set_visible(self.show_simulation_mesh);
        }
    }

    pub fn update_job_origin(&mut self, x: f32, y: f32, z: f32) {
        self.job_origin.translation.vector.x = x;
        self.job_origin.translation.vector.y = y;
        self.job_origin.translation.vector.z = z;
    }

    pub fn set_current_time_step(&mut self, time_step: usize) {
        self.current_time_step = time_step.min(self.max_time_steps);
        self.update_simulation();
    }
}

fn get_task_color(task_index: usize) -> [f32; 3] {
    const COLORS: [[f32; 3]; 6] = [
        [1.0, 0.0, 0.3],  // Red
        [0.0, 1.0, 0.0],  // Green
        [0.0, 0.0, 1.0],  // Blue
        [1.0, 1.0, 0.0],  // Yellow
        [1.0, 0.0, 1.0],  // Magenta
        [0.0, 1.0, 1.0],  // Cyan
    ];
    COLORS[task_index % COLORS.len()]
}
pub fn handle_ui(app_state: &mut AppState, ui: &mut UiCell) -> bool {
    let ids = &app_state.ids;
    let mut ui_changed = false;
    let mut toggle_mesh = false;
    let mut toggle_stock_mesh = false;
    let mut toggle_keypoints = false;
    let mut toggle_keypoint_lines = false;
    let mut toggle_simulation_mesh = false;
    let mut new_is_playing = app_state.is_playing;
    let mut new_job_origin = app_state.job_origin;
    let mut new_time_step = app_state.current_time_step;

    // Process button
    for _click in widget::Button::new()
        .top_left_with_margin(20.0)
        .w_h(100.0, 30.0)
        .label("Process")
        .set(ids.process_button, ui)
    {
        if let Err(e) = app_state.cam_job.lock().unwrap().build() {
            eprintln!("Failed to build CAM job: {}", e);
        } 
        ui_changed = true;
    }

    // Play/Pause button
    for _click in widget::Button::new()
        .right_from(ids.process_button, 10.0)
        .w_h(100.0, 30.0)
        .label(if app_state.is_playing { "Pause" } else { "Play" })
        .set(ids.play_pause_button, ui)
    {
        new_is_playing = !app_state.is_playing;
        ui_changed = true;
    }

    // Toggle Mesh button
    for _click in widget::Button::new()
        .down_from(ids.process_button, 10.0)
        .w_h(100.0, 30.0)
        .label(if app_state.show_mesh { "Hide Mesh" } else { "Show Mesh" })
        .set(ids.toggle_mesh_button, ui)
    {
        toggle_mesh = true;
        ui_changed = true;
    }

    // Toggle Stock Mesh button
    for _click in widget::Button::new()
        .right_from(ids.toggle_mesh_button, 10.0)
        .w_h(120.0, 30.0)
        .label(if app_state.show_stock_mesh { "Hide Stock Mesh" } else { "Show Stock Mesh" })
        .set(ids.toggle_stock_mesh_button, ui)
    {
        toggle_stock_mesh = true;
        ui_changed = true;
    }

    // Toggle Keypoints button
    for _click in widget::Button::new()
        .down_from(ids.toggle_mesh_button, 10.0)
        .w_h(100.0, 30.0)
        .label(if app_state.show_keypoints { "Hide Keypoints" } else { "Show Keypoints" })
        .set(ids.toggle_keypoints_button, ui)
    {
        toggle_keypoints = true;
        ui_changed = true;
    }

    // Toggle Keypoint Lines button
    for _click in widget::Button::new()
        .right_from(ids.toggle_keypoints_button, 10.0)
        .w_h(150.0, 30.0)
        .label(if app_state.show_keypoint_lines { "Hide Keypoint Lines" } else { "Show Keypoint Lines" })
        .set(ids.toggle_keypoint_lines_button, ui)
    {
        toggle_keypoint_lines = true;
        ui_changed = true;
    }

    // Display current values
    widget::Text::new(&format!("Layers: {}", app_state.num_layers))
        .down_from(ids.toggle_keypoint_lines_button, 10.0)
        .color(color::BLACK)
        .set(ids.layers_text, ui);

    widget::Text::new(&format!("Current Layer: {}", app_state.current_layer))
        .down_from(ids.layers_text, 5.0)
        .color(color::BLACK)
        .set(ids.current_layer_text, ui);

    widget::Text::new(&format!("Rays: {}", app_state.num_rays))
        .down_from(ids.current_layer_text, 5.0)
        .color(color::BLACK)
        .set(ids.rays_text, ui);

    widget::Text::new(&format!("Ray Length: {:.2}", app_state.ray_length))
        .down_from(ids.rays_text, 5.0)
        .color(color::BLACK)
        .set(ids.ray_length_text, ui);

    widget::Text::new(&format!("Animation Speed: {:.2}", app_state.animation_speed))
        .down_from(ids.ray_length_text, 5.0)
        .color(color::BLACK)
        .set(ids.animation_speed_text, ui);

    // Job Origin controls
    widget::Text::new(&format!("Origin X: {:.2}", app_state.job_origin.translation.vector.x))
        .down_from(ids.animation_speed_text, 10.0)
        .color(color::BLACK)
        .set(ids.origin_x_text, ui);

    for value in widget::Slider::new(app_state.job_origin.translation.vector.x, -1.0, 1.0)
        .down_from(ids.origin_x_text, 5.0)
        .w_h(200.0, 30.0)
        .set(ids.origin_x_slider, ui)
    {
        new_job_origin.translation.vector.x = value;
        ui_changed = true;
    }

    // Similar controls for Origin Y and Z...

    // Time step control
    widget::Text::new(&format!("Time Step: {}/{}", app_state.current_time_step, app_state.max_time_steps))
        .down_from(ids.origin_z_slider, 10.0)
        .color(color::BLACK)
        .set(ids.time_step_text, ui);

    for value in widget::Slider::new(app_state.current_time_step as f32, 0.0, app_state.max_time_steps as f32)
        .down_from(ids.time_step_text, 5.0)
        .w_h(200.0, 30.0)
        .set(ids.time_step_slider, ui)
    {
        new_time_step = value as usize;
        ui_changed = true;
    }

    // Toggle Simulation Mesh button
    for _click in widget::Button::new()
        .down_from(ids.time_step_slider, 10.0)
        .w_h(150.0, 30.0)
        .label(if app_state.show_simulation_mesh { "Hide Simulation Mesh" } else { "Show Simulation Mesh" })
        .set(ids.toggle_simulation_mesh_button, ui)
    {
        toggle_simulation_mesh = true;
        ui_changed = true;
    }

    // Apply all changes at once
    if ui_changed {
        if toggle_mesh {
            app_state.toggle_mesh_visibility();
        }
        if toggle_stock_mesh {
            app_state.toggle_stock_mesh_visibility();
        }
        if toggle_keypoints {
            app_state.toggle_keypoints_visibility();
        }
        if toggle_keypoint_lines {
            app_state.toggle_keypoint_lines_visibility();
        }
        if toggle_simulation_mesh {
            app_state.toggle_simulation_mesh_visibility();
        }
        app_state.is_playing = new_is_playing;
        app_state.job_origin = new_job_origin;
        app_state.set_current_time_step(new_time_step);
    }

    ui_changed
}