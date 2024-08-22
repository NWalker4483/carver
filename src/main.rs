mod errors;
mod prelude;
mod tasks;
mod cam_job;
mod app_state;
mod tool;
mod stl_operations;

use app_state::{AppState, handle_ui};
use stl_operations::{center_and_scale_mesh, load_stl, mesh_to_kiss3d};
use cam_job::CAMJOB;
use tool::Tool;
use kiss3d::nalgebra::{Vector3, Point3};
use kiss3d::window::Window;
use kiss3d::light::Light;
use tasks::*;
use std::rc::Rc;
use std::{cell::RefCell, path::Path};
use std::env;
use anyhow::Result;

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

    let mut cam_job = CAMJOB::new();
    cam_job.set_mesh(mesh.clone())?;

    // Initialize tools
    cam_job.add_tool(Tool::new(0, "End Mill 6mm".to_string(), &mut window, 0.05, 0.006));
    cam_job.add_tool(Tool::new(1, "Ball Mill 4mm".to_string(), &mut window, 0.04, 0.004));

    let mut stock_mesh = window.add_mesh(
        Rc::new(RefCell::new(mesh_to_kiss3d(cam_job.get_stock_mesh().unwrap()))),
        Vector3::new(1.0, 1.0, 1.0)
    );
    stock_mesh.set_color(0.5, 0.5, 0.5);
    stock_mesh.set_lines_width(1.0);
    stock_mesh.set_surface_rendering_activation(false);

    cam_job.add_task(Box::new(MultiContourTrace::new(
        Point3::new(0.0, 0.0, min_z),
        Point3::new(0.0, 0.0, max_z),
        50,
        200,
        // 0, // tool_id for End Mill 6mm
    )));

    cam_job.add_task(Box::new(CircularClearing::new(
        Point3::new(0.0, 0.0, min_z),
        Point3::new(0.0, 0.0, max_z),
        50,
        75.0,
        50,
        5.,
        0.001,
        // 1, // tool_id for Ball Mill 4mm
    )));

    // Initialize AppState
    let mut app_state = {
        let mut ui = window.conrod_ui_mut().set_widgets();
        AppState::new(mesh.clone(), cam_job, stock_mesh, &mut ui)
    };

    while window.render() {
        {
            let mut ui = window.conrod_ui_mut().set_widgets();
            handle_ui(&mut app_state, &mut ui);
        }

        if app_state.show_keypoint_lines {
            app_state.draw_keypoint_lines(&mut window);
        }

        if app_state.is_playing {
            app_state.animate();
        }

        // Update mesh visibility
        c.set_visible(app_state.show_mesh);

        // Update stock mesh visibility
        app_state.stock_mesh.set_visible(app_state.show_stock_mesh);

        // Update tool visibility
        let cam_job = app_state.cam_job.lock().unwrap();
        for tool_id in 0..2 {  // Assuming we have 2 tools
            if let Some(tool) = cam_job.get_tool(tool_id) {
                tool.set_visible(app_state.is_playing);
            }
        }
    }

    Ok(())
}