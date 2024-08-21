use std::sync::Arc;
use std::sync::Mutex;
use kiss3d::window::Window;
use kiss3d::scene::SceneNode;
use kiss3d::nalgebra::{Point3, Vector3, Translation3, UnitQuaternion, Isometry3};
use kiss3d::conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget, UiCell};
use kiss3d::conrod::widget_ids;
use stl_io::IndexedMesh;
use crate::cam_job::{CAMJOB, Keypoint};

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
    pub cylinder: SceneNode,
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
    pub fn new(mesh: IndexedMesh, cylinder: SceneNode, cam_job: CAMJOB, stock_mesh: SceneNode, ui: &mut UiCell) -> Self {
        AppState {
            mesh: mesh.clone(),
            cam_job: Arc::new(Mutex::new(cam_job)),
            num_layers: 40,
            num_rays: 100,
            ray_length: 0.9,
            is_playing: false,
            current_layer: 0,
            animation_speed: 1.0,
            cylinder,
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
            self.cylinder.set_local_translation(Translation3::from(transformed_position.coords));
            let rotation = UnitQuaternion::rotation_between(&Vector3::z(), &keypoint.normal)
                .unwrap_or(UnitQuaternion::identity());
            self.cylinder.set_local_rotation(rotation);
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
        // self.update_tool_position();
        // Additional logic for updating workpiece state could go here
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
        let cam_job = self.cam_job.lock().unwrap();
        if let Some(tool_position) = cam_job.get_tool_position_at_time_step(self.current_time_step) {
            let transformed_position = self.job_origin * tool_position;
            self.cylinder.set_local_translation(Translation3::from(transformed_position.coords));
            // You might also want to update the tool orientation here
        }
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

pub fn handle_ui( app_state: &mut AppState, ui: &mut UiCell) -> bool {
    let ids = &app_state.ids;
    let mut ui_changed = false;

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
        app_state.is_playing = !app_state.is_playing;
        app_state.cylinder.set_visible(app_state.is_playing);
        ui_changed = true;
    }

    // Toggle Mesh button
    for _click in widget::Button::new()
        .down_from(ids.process_button, 10.0)
        .w_h(100.0, 30.0)
        .label(if app_state.show_mesh { "Hide Mesh" } else { "Show Mesh" })
        .set(ids.toggle_mesh_button, ui)
    {
        app_state.show_mesh = !app_state.show_mesh;
        ui_changed = true;
    }

    // Toggle Stock Mesh button
    for _click in widget::Button::new()
        .right_from(ids.toggle_mesh_button, 10.0)
        .w_h(120.0, 30.0)
        .label(if app_state.show_stock_mesh { "Hide Stock Mesh" } else { "Show Stock Mesh" })
        .set(ids.toggle_stock_mesh_button, ui)
    {
        app_state.show_stock_mesh = !app_state.show_stock_mesh;
        app_state.stock_mesh.set_visible(app_state.show_stock_mesh);
        ui_changed = true;
    }

    // Toggle Keypoints button
    for _click in widget::Button::new()
        .down_from(ids.toggle_mesh_button, 10.0)
        .w_h(100.0, 30.0)
        .label(if app_state.show_keypoints { "Hide Keypoints" } else { "Show Keypoints" })
        .set(ids.toggle_keypoints_button, ui)
    {
        app_state.show_keypoints = !app_state.show_keypoints;
        for sphere in &mut app_state.keypoint_spheres {
            sphere.set_visible(app_state.show_keypoints);
        }
        ui_changed = true;
    }

    // Toggle Keypoint Lines button
    for _click in widget::Button::new()
        .right_from(ids.toggle_keypoints_button, 10.0)
        .w_h(150.0, 30.0)
        .label(if app_state.show_keypoint_lines { "Hide Keypoint Lines" } else { "Show Keypoint Lines" })
        .set(ids.toggle_keypoint_lines_button, ui)
    {
        app_state.show_keypoint_lines = !app_state.show_keypoint_lines;
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
        app_state.job_origin.translation.vector.x = value;
        ui_changed = true;
    }

    widget::Text::new(&format!("Origin Y: {:.2}", app_state.job_origin.translation.vector.y))
        .down_from(ids.origin_x_slider, 10.0)
        .color(color::BLACK)
        .set(ids.origin_y_text, ui);

    for value in widget::Slider::new(app_state.job_origin.translation.vector.y, -1.0, 1.0)
        .down_from(ids.origin_y_text, 5.0)
        .w_h(200.0, 30.0)
        .set(ids.origin_y_slider, ui)
    {
        app_state.job_origin.translation.vector.y = value;
        ui_changed = true;
    }

    widget::Text::new(&format!("Origin Z: {:.2}", app_state.job_origin.translation.vector.z))
        .down_from(ids.origin_y_slider, 10.0)
        .color(color::BLACK)
        .set(ids.origin_z_text, ui);

    for value in widget::Slider::new(app_state.job_origin.translation.vector.z, -1.0, 1.0)
        .down_from(ids.origin_z_text, 5.0)
        .w_h(200.0, 30.0)
        .set(ids.origin_z_slider, ui)
    {
        app_state.job_origin.translation.vector.z = value; 
        ui_changed = true;
    }

    // Time step display and control
    widget::Text::new(&format!("Time Step: {}/{}", app_state.current_time_step, app_state.max_time_steps))
        .down_from(ids.origin_z_slider, 10.0)
        .color(color::BLACK)
        .set(ids.time_step_text, ui);

    for value in widget::Slider::new(app_state.current_time_step as f32, 0.0, app_state.max_time_steps as f32)
        .down_from(ids.time_step_text, 5.0)
        .w_h(200.0, 30.0)
        .set(ids.time_step_slider, ui)
    {
        app_state.current_time_step = value as usize;
        // app_state.update_simulation();
        ui_changed = true;
    }

    // Toggle Simulation Mesh button
    for _click in widget::Button::new()
        .down_from(ids.time_step_slider, 10.0)
        .w_h(150.0, 30.0)
        .label(if app_state.show_simulation_mesh { "Hide Simulation Mesh" } else { "Show Simulation Mesh" })
        .set(ids.toggle_simulation_mesh_button, ui)
    {
        app_state.show_simulation_mesh = !app_state.show_simulation_mesh;
        if app_state.show_simulation_mesh {
            app_state.generate_simulation_mesh();
        }
        if let Some(sim_mesh) = &mut app_state.simulation_mesh {
            sim_mesh.set_visible(app_state.show_simulation_mesh);
        }
        ui_changed = true;
    }

    ui_changed
}


// Add these trait implementations at the end of the file

impl CAMJOB {
pub fn update_to_time_step(&mut self, time_step: usize) {
// Implement the logic to update the CAM job to a specific time step
println!("Updating CAM job to time step: {}", time_step);
}

pub fn get_tool_position_at_time_step(&self, time_step: usize) -> Option<Point3<f32>> {
// Implement the logic to get the tool position at a specific time step
println!("Getting tool position at time step: {}", time_step);
Some(Point3::new(0.0, 0.0, 0.0)) // Placeholder return value
}

pub fn create_simulation_mesh(&self, time_step: usize) -> SceneNode {
// Implement the logic to create a new simulation mesh
println!("Creating simulation mesh for time step: {}", time_step);
// Placeholder: You'll need to actually create and return a SceneNode here
unimplemented!("create_simulation_mesh not yet implemented")
}

pub fn update_simulation_mesh(&self, mesh: &mut SceneNode, time_step: usize) {
// Implement the logic to update an existing simulation mesh
println!("Updating simulation mesh for time step: {}", time_step);
}
}