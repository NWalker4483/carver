use kiss3d::window::Window;
use kiss3d::scene::SceneNode;
use kiss3d::nalgebra::{Point3, Vector3, Translation3, UnitQuaternion};
use kiss3d::conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};
use kiss3d::conrod::widget_ids;
use crate::cam_job::{CAMJOB, Keypoint};
use crate::stl_operations::IndexedMesh;

widget_ids! {
    pub struct Ids {
        process_button,
        play_pause_button,
        toggle_mesh_button,
        toggle_keypoints_button,
        layers_text,
        current_layer_text,
        rays_text,
        ray_length_text,
        animation_speed_text,
    }
}

pub struct AppState {
    pub mesh: IndexedMesh,
    pub cam_job: CAMJOB,
    pub num_layers: usize,
    pub num_rays: usize,
    pub ray_length: f32,
    pub is_playing: bool,
    pub current_layer: usize,
    pub animation_speed: f32,
    pub cylinder: SceneNode,
    pub show_mesh: bool,
    pub show_keypoints: bool,
    pub current_keypoint: usize,
    ids: Ids,
}

impl AppState {
    pub fn new(mesh: IndexedMesh, cylinder: SceneNode, cam_job: CAMJOB, ui: &mut kiss3d::conrod::UiCell) -> Self {
        AppState {
            mesh: mesh.clone(),
            cam_job,
            num_layers: 40,
            num_rays: 100,
            ray_length: 0.1,
            is_playing: false,
            current_layer: 0,
            animation_speed: 1.0,
            cylinder,
            show_mesh: true,
            show_keypoints: true,
            current_keypoint: 0,
            ids: Ids::new(ui.widget_id_generator()),
        }
    }

    pub fn animate(&mut self) {
        let keypoints = self.cam_job.gather_keypoints();
        if !keypoints.is_empty() {
            self.current_keypoint = (self.current_keypoint + 1) % keypoints.len();
            let keypoint = &keypoints[self.current_keypoint];
            self.cylinder.set_local_translation(Translation3::from(keypoint.position.coords));
            // You might want to add rotation based on the normal here
            let rotation = UnitQuaternion::rotation_between(&Vector3::z(), &keypoint.normal)
                .unwrap_or(UnitQuaternion::identity());
            self.cylinder.set_local_rotation(rotation);
        }
    }

    pub fn draw_keypoints(&self, window: &mut Window) {
        let keypoints = self.cam_job.gather_keypoints();
        for keypoint in keypoints {
            let end_point = keypoint.position + keypoint.normal * self.ray_length;
            window.draw_line(&keypoint.position, &end_point, &Point3::new(1.0, 1.0, 0.0));
        }
    }
}

pub fn handle_ui(window: &mut Window, app_state: &mut AppState) -> bool {
    let ids = &app_state.ids;
    let mut ui = window.conrod_ui_mut().set_widgets();
    let mut ui_changed = false;

    // Process button
    for _click in widget::Button::new()
        .top_left_with_margin(20.0)
        .w_h(100.0, 30.0)
        .label("Process")
        .set(ids.process_button, &mut ui)
    {
        if let Err(e) = app_state.cam_job.build() {
            eprintln!("Failed to build CAM job: {}", e);
        }
        ui_changed = true;
    }

    // Play/Pause button
    for _click in widget::Button::new()
        .right_from(ids.process_button, 10.0)
        .w_h(100.0, 30.0)
        .label(if app_state.is_playing { "Pause" } else { "Play" })
        .set(ids.play_pause_button, &mut ui)
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
        .set(ids.toggle_mesh_button, &mut ui)
    {
        app_state.show_mesh = !app_state.show_mesh;
        ui_changed = true;
    }

    // Toggle Keypoints button
    for _click in widget::Button::new()
        .right_from(ids.toggle_mesh_button, 10.0)
        .w_h(100.0, 30.0)
        .label(if app_state.show_keypoints { "Hide Keypoints" } else { "Show Keypoints" })
        .set(ids.toggle_keypoints_button, &mut ui)
    {
        app_state.show_keypoints = !app_state.show_keypoints;
        ui_changed = true;
    }

    // Display current values
    widget::Text::new(&format!("Layers: {}", app_state.num_layers))
        .down_from(ids.toggle_mesh_button, 10.0)
        .color(color::BLACK)
        .set(ids.layers_text, &mut ui);

    widget::Text::new(&format!("Current Layer: {}", app_state.current_layer))
        .down_from(ids.layers_text, 5.0)
        .color(color::BLACK)
        .set(ids.current_layer_text, &mut ui);

    widget::Text::new(&format!("Rays: {}", app_state.num_rays))
        .down_from(ids.current_layer_text, 5.0)
        .color(color::BLACK)
        .set(ids.rays_text, &mut ui);

    widget::Text::new(&format!("Ray Length: {:.2}", app_state.ray_length))
        .down_from(ids.rays_text, 5.0)
        .color(color::BLACK)
        .set(ids.ray_length_text, &mut ui);

    widget::Text::new(&format!("Animation Speed: {:.2}", app_state.animation_speed))
        .down_from(ids.ray_length_text, 5.0)
        .color(color::BLACK)
        .set(ids.animation_speed_text, &mut ui);

    ui_changed
}
