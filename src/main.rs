use cam_job::CAMJOB;
use errors::CAMError;
use stl_io::IndexedMesh;
use kiss3d::nalgebra::{Vector3, Point3};
use kiss3d::window::Window;
use kiss3d::light::Light;
use tasks::MultiContourTrace;
use std::rc::Rc;
use std::{cell::RefCell, path::Path};
use std::env;
use anyhow::Result;

mod errors;
mod prelude;
mod tasks;
mod cam_job;
mod app_state;
mod stl_operations;

use app_state::{AppState, handle_ui};
use stl_operations::{center_and_scale_mesh, load_stl, mesh_to_kiss3d};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <stl_file>", args[0]);
        std::process::exit(1);
    }
    
    let stl_file = &args[1];
    let filename = Path::new(stl_file);
    let mut mesh = load_stl(filename)?;
    let (min_z, max_z) = center_and_scale_mesh(&mut mesh);
    
    let mut window = Window::new("STL Viewer with Keypoints");
    let mut c = window.add_mesh(Rc::new(RefCell::new(mesh_to_kiss3d(&mesh))), Vector3::new(1.0, 1.0, 1.0));
    c.set_color(0.8, 0.8, 0.8);
    c.set_lines_width(1.0);
    c.set_surface_rendering_activation(false);
    window.set_light(Light::StickToCamera);
    
    let mut cylinder = window.add_cylinder(0.02, 0.02);
    cylinder.set_color(1.0, 0.0, 0.0);
    cylinder.set_visible(false);
    
    // Initialize AppState outside the loop
    let mut app_state = {
        let mut ui = window.conrod_ui_mut().set_widgets();
        let mut cam_job = CAMJOB::new();
        cam_job.set_mesh(mesh.clone());
        // Automatically add the task for testing
        cam_job.add_task(Box::new(MultiContourTrace::new(
            Point3::new(0.0, 0.0, min_z),  // start_position
            Point3::new(0.0, 0.0, max_z),  // end_position
            20,  // num_layers
            200,  // num_rays
        )));
        AppState::new(mesh.clone(), cylinder, cam_job, &mut ui)
    };

    while window.render() {
        handle_ui(&mut window, &mut app_state);
        
        if app_state.show_keypoints {
            app_state.draw_keypoints(&mut window);
        }
        
        if app_state.is_playing {
            app_state.animate();
        }
        
        // Update mesh visibility
        c.set_visible(app_state.show_mesh);
    }
    
    Ok(())
}