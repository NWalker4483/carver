use kiss3d::light::Light;
use kiss3d::window::Window;
use kiss3d::nalgebra::{Point2, Point3, Vector3, UnitQuaternion};
use kiss3d::text::Font;
use std::path::Path;
use stl_io::IndexedMesh;
use std::fs::File;
use std::env;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{bail, Result};

// Configuration parameters
const NUM_LAYERS: usize = 20;
const ANIMATION_SPEED: f32 = 0.5;
const ROTATION_SPEED: f32 = 0.005;
const TOOL_RADIUS: f32 = 0.01;
const TOOL_HEIGHT: f32 = 0.05;

// Debug configuration
const DEBUG_MODE: bool = true;
const VERBOSE_LOGGING: bool = true;
const FONT_SIZE: f32 = 20.0;

fn load_stl(filename: &Path) -> Result<IndexedMesh> {
    let mut file = File::open(filename)?;
    Ok(stl_io::read_stl(&mut file)?)
}



fn get_bounds(mesh: &IndexedMesh) -> (Vector3<f32>, Vector3<f32>) {
    let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
    for v in &mesh.vertices {
        min.x = min.x.min(v[0]);
        min.y = min.y.min(v[1]);
        min.z = min.z.min(v[2]);
        max.x = max.x.max(v[0]);
        max.y = max.y.max(v[1]);
        max.z = max.z.max(v[2]);
    }
    (min, max)
}

fn center_and_scale_mesh(mesh: &mut IndexedMesh) -> Result<(f32, f32)> {
    let (min, max) = get_bounds(mesh);
    let center = (min + max) / 2.0;
    let size = max - min;
    
    // Avoid division by zero
    let scale = if size.x.abs() > 1e-6 && size.y.abs() > 1e-6 && size.z.abs() > 1e-6 {
        1.0 / size.amax()
    } else {
        bail!("Mesh has zero size in at least one dimension");
    };

    // // Apply transformation to vertices
    // for v in &mut mesh.vertices {
    //     *v = (*v - center) * scale;
    // }

    let min_z = (min.z - center.z) * scale;
    let max_z = (max.z - center.z) * scale;

    if min_z >= max_z {
        bail!("Invalid mesh: min_z >= max_z after scaling");
    }

    Ok((min_z, max_z))
}

fn generate_contour(mesh: &IndexedMesh, height: f32) -> Vec<Point3<f32>> {
    let mut contour = Vec::new();
    for face in &mesh.faces {
        let v1 = Point3::new(mesh.vertices[face.vertices[0] as usize][0], 
                             mesh.vertices[face.vertices[0] as usize][1], 
                             mesh.vertices[face.vertices[0] as usize][2]);
        let v2 = Point3::new(mesh.vertices[face.vertices[1] as usize][0], 
                             mesh.vertices[face.vertices[1] as usize][1], 
                             mesh.vertices[face.vertices[1] as usize][2]);
        let v3 = Point3::new(mesh.vertices[face.vertices[2] as usize][0], 
                             mesh.vertices[face.vertices[2] as usize][1], 
                             mesh.vertices[face.vertices[2] as usize][2]);

        let points = vec![v1, v2, v3];
        let mut intersections = Vec::new();

        // Check for intersections between the plane at 'height' and each edge of the triangle
        for i in 0..3 {
            let p1 = points[i];
            let p2 = points[(i + 1) % 3];

            if (p1.z <= height && p2.z > height) || (p1.z > height && p2.z <= height) {
                let t = (height - p1.z) / (p2.z - p1.z);
                let intersection = p1 + (p2 - p1) * t;
                intersections.push(intersection);
            }
        }

        contour.extend(intersections);
    }
    contour
}

fn generate_layers(mesh: &IndexedMesh, start: f32, end: f32, cuts: usize) -> Vec<Vec<Point3<f32>>> {
    let step = (end - start) / cuts as f32;
    (0..=cuts).map(|i| {
        let layer_height = start + i as f32 * step;
        generate_contour(mesh, layer_height)
    }).collect()
}

fn mesh_to_kiss3d(mesh: &IndexedMesh) -> kiss3d::resource::Mesh {
    let vertices: Vec<Point3<f32>> = mesh.vertices.iter()
        .map(|v| Point3::new(v[0], v[1], v[2]))
        .collect();
    
    let indices: Vec<Point3<u16>> = mesh.faces.iter()
        .map(|f| Point3::new(f.vertices[0] as u16, f.vertices[1] as u16, f.vertices[2] as u16))
        .collect();

    kiss3d::resource::Mesh::new(vertices, indices, None, None, false)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <stl_file>", args[0]);
        std::process::exit(1);
    }
    let stl_file = &args[1];

    let filename = Path::new(stl_file);
    let mut mesh = load_stl(filename)?;
    let (min_z, max_z) = center_and_scale_mesh(&mut mesh)?;

    if VERBOSE_LOGGING {
        println!("Loaded STL file: {}", stl_file);
        println!("Mesh bounds after scaling: min_z = {}, max_z = {}", min_z, max_z);
    }

    let mut window = Window::new("Carver");
    let mut c = window.add_mesh(Rc::new(RefCell::new(mesh_to_kiss3d(&mesh))), Vector3::new(1.0, 1.0, 1.0));
    c.set_color(0.8, 0.8, 0.8);
    c.set_lines_width(1.0);
    c.set_surface_rendering_activation(false);

    window.set_light(Light::StickToCamera);

    let layers = generate_layers(&mesh, min_z, max_z, NUM_LAYERS);

    if VERBOSE_LOGGING {
        println!("Generated {} layers", layers.len());
        for (i, layer) in layers.iter().enumerate() {
            println!("Layer {}: {} points", i, layer.len());
        }
    }

    let mut tool = window.add_cylinder(TOOL_RADIUS, TOOL_HEIGHT);
    tool.set_color(1.0, 0.0, 0.0);

    let mut t = 0.0;
    let mut current_layer = 0;
    let mut current_point = 0;
    let mut frame_count = 0;

    while window.render() {
        frame_count += 1;
        t += ANIMATION_SPEED;

        if VERBOSE_LOGGING && frame_count % 100 == 0 {
            println!("Frame: {}, Time: {:.2}", frame_count, t);
        }

        // Draw layers
        for (layer_index, layer) in layers.iter().enumerate() {
            for i in 0..layer.len() {
                let start = layer[i];
                let end = layer[(i + 1) % layer.len()];
                window.draw_line(&start, &end, &Point3::new(0.0, 0.0, 1.0));
            }

            if VERBOSE_LOGGING && frame_count % 1000 == 0 {
                println!("Drew layer {}: {} points", layer_index, layer.len());
            }
        }

        if !layers[current_layer].is_empty() {
            let target_point = layers[current_layer][current_point];
            tool.set_local_translation(target_point.into());

            if VERBOSE_LOGGING {
                println!("Tool position: {:?}", target_point);
            }

            if current_point < layers[current_layer].len() - 1 {
                let next_point = layers[current_layer][(current_point + 1) % layers[current_layer].len()];
                let direction = next_point - target_point;
                if direction.magnitude() > 1e-6 {
                    let rotation = UnitQuaternion::rotation_between(&Vector3::z(), &direction.normalize())
                        .unwrap_or(UnitQuaternion::identity());
                    tool.set_local_rotation(rotation);

                    if VERBOSE_LOGGING {
                        println!("Tool rotation: {:?}", rotation);
                    }
                }
            }

            current_point = (current_point + 1) % layers[current_layer].len();

            if current_point == 0 {
                current_layer = (current_layer + 1) % layers.len();
                if VERBOSE_LOGGING {
                    println!("Moving to next layer: {}", current_layer);
                }
            }
        }
        

        if DEBUG_MODE {
            let font = Font::default();
            window.draw_text(
                &format!("File: {}", stl_file),
                &Point2::new(-0.9, 0.9),
                FONT_SIZE,
                &font,
                &Point3::new(1.0, 1.0, 1.0),
            );
            window.draw_text(
                &format!("Time: {:.2}", t),
                &Point2::new(-0.9, 0.8),
                FONT_SIZE,
                &font,
                &Point3::new(1.0, 1.0, 1.0),
            );
            window.draw_text(
                &format!("Layer: {}/{}", current_layer + 1, layers.len()),
                &Point2::new(-0.9, 0.7),
                FONT_SIZE,
                &font,
                &Point3::new(1.0, 1.0, 1.0),
            );
            window.draw_text(
                &format!("Height Range: {:.2} to {:.2}", min_z, max_z),
                &Point2::new(-0.9, 0.6),
                FONT_SIZE,
                &font,
                &Point3::new(1.0, 1.0, 1.0),
            );
        }

        // Check for exit condition (e.g., pressing 'Q')
        // if window.get_key(kiss3d::event::Key::Q) == kiss3d::event::Action::Release {
        //     if VERBOSE_LOGGING {
        //         println!("Exit key pressed. Terminating program.");
        //     }
        //     break;
        // }
    }

    Ok(())
}