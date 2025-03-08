//! # Gerber to USDZ Converter Library
//!
//! This library converts Gerber files (standard format for PCB manufacturing)
//! to USDZ 3D models (for AR/VR visualization) or OBJ models for general use.
//!
//! ## Module Structure
//!
//! - `gerber`: Module for parsing Gerber files
//!   - `types.rs`: Definitions of Gerber file structures and commands
//!   - `parse.rs`: Parser for Gerber file format using nom
//!
//! - `intermediate`: Module for 3D model representation
//!   - `model.rs`: Definitions of 3D mesh structures (vertices, faces, etc.)
//!
//! - `usdz`: Module for USDZ file generation
//!   - `export.rs`: Export functions for USDZ and OBJ formats
//!
//! ## Workflow
//!
//! 1. Scan directory for Gerber files and categorize them by layer type
//! 2. Parse each Gerber file into structured commands
//! 3. Convert each layer to a 3D mesh based on its type
//! 4. Combine meshes into a complete PCB model
//! 5. Export to USDZ or OBJ format based on user preference

pub mod gerber;
pub mod intermediate;
pub mod usdz;

use intermediate::model::{LayerType, Mesh, PCBModel, Units};
use std::fs;
use std::path::{Path, PathBuf};

/// Process Gerber files to create a 3D PCB model.
///
/// This function:
/// 1. Reads each Gerber file for different PCB layers (Edge Cuts, Copper, Silkscreen, etc.)
/// 2. Parses the Gerber commands
/// 3. Converts each layer to a 3D mesh
/// 4. Combines meshes into a complete PCB model
///
/// # Arguments
///
/// * `input_dir` - Directory containing Gerber files
/// * `thickness` - PCB thickness in mm
///
/// # Returns
///
/// * `Result<PCBModel, String>` - The complete PCB model on success, or an error message
pub fn process_gerber_files(input_dir: &str, thickness: f64) -> Result<PCBModel, String> {
    let input_path = Path::new(input_dir);

    // Check if the input directory exists
    if !input_path.exists() || !input_path.is_dir() {
        return Err(format!("Input directory does not exist: {}", input_dir));
    }

    // Create a PCB model
    let mut pcb_model = PCBModel {
        meshes: Vec::new(),
        units: Units::Millimeters, // Default to mm
    };

    // Find and process Gerber files
    let entries =
        fs::read_dir(input_path).map_err(|e| format!("Error reading directory: {}", e))?;

    // Collect file paths by layer type
    let mut edge_cuts_file: Option<PathBuf> = None;
    let mut top_copper_file: Option<PathBuf> = None;
    let mut bottom_copper_file: Option<PathBuf> = None;
    let mut top_silk_file: Option<PathBuf> = None;
    let mut bottom_silk_file: Option<PathBuf> = None;

    // First pass: categorize files by their likely layer type
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "gbr" || ext == "GBR" {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_lowercase();

                    // Categorize by common naming conventions
                    if file_name.contains("edge")
                        || file_name.contains("outline")
                        || file_name.contains("cuts")
                    {
                        edge_cuts_file = Some(path.clone());
                    } else if file_name.contains("f.cu")
                        || file_name.contains("f_cu")
                        || file_name.contains("top.cu")
                    {
                        top_copper_file = Some(path.clone());
                    } else if file_name.contains("b.cu")
                        || file_name.contains("b_cu")
                        || file_name.contains("bottom.cu")
                    {
                        bottom_copper_file = Some(path.clone());
                    } else if file_name.contains("f.silk")
                        || file_name.contains("f_silk")
                        || file_name.contains("top.silk")
                    {
                        top_silk_file = Some(path.clone());
                    } else if file_name.contains("b.silk")
                        || file_name.contains("b_silk")
                        || file_name.contains("bottom.silk")
                    {
                        bottom_silk_file = Some(path.clone());
                    }
                }
            }
        }
    }

    // Process Edge Cuts layer first (required for PCB outline)
    if let Some(path) = edge_cuts_file {
        println!("Processing Edge Cuts layer: {:?}", path);
        let edge_cuts_commands = read_and_parse_gerber(path.to_str().unwrap())?;
        let edge_cuts_mesh = build_edge_cuts_mesh(&edge_cuts_commands, Some(thickness))?;
        pcb_model.meshes.push(edge_cuts_mesh);
    } else {
        return Err("Edge Cuts layer not found. This is required for the PCB outline.".to_string());
    }

    // Process copper layers
    if let Some(path) = top_copper_file {
        println!("Processing top copper layer: {:?}", path);
        if let Ok(commands) = read_and_parse_gerber(path.to_str().unwrap()) {
            match build_copper_mesh(&commands, true, Some(thickness)) {
                Ok(mesh) => {
                    println!(
                        "Top copper mesh created with {} vertices and {} faces",
                        mesh.vertices.len(),
                        mesh.faces.len()
                    );
                    pcb_model.meshes.push(mesh);
                }
                Err(e) => println!("Warning: Failed to create top copper mesh: {}", e),
            }
        }
    }

    if let Some(path) = bottom_copper_file {
        println!("Processing bottom copper layer: {:?}", path);
        if let Ok(commands) = read_and_parse_gerber(path.to_str().unwrap()) {
            match build_copper_mesh(&commands, false, Some(thickness)) {
                Ok(mesh) => {
                    println!(
                        "Bottom copper mesh created with {} vertices and {} faces",
                        mesh.vertices.len(),
                        mesh.faces.len()
                    );
                    pcb_model.meshes.push(mesh);
                }
                Err(e) => println!("Warning: Failed to create bottom copper mesh: {}", e),
            }
        }
    }

    // Process silkscreen layers
    if let Some(path) = top_silk_file {
        println!("Processing top silkscreen layer: {:?}", path);
        if let Ok(commands) = read_and_parse_gerber(path.to_str().unwrap()) {
            match build_silkscreen_mesh(&commands, true, Some(thickness)) {
                Ok(mesh) => {
                    println!(
                        "Top silkscreen mesh created with {} vertices and {} faces",
                        mesh.vertices.len(),
                        mesh.faces.len()
                    );
                    pcb_model.meshes.push(mesh);
                }
                Err(e) => println!("Warning: Failed to create top silkscreen mesh: {}", e),
            }
        }
    }

    if let Some(path) = bottom_silk_file {
        println!("Processing bottom silkscreen layer: {:?}", path);
        if let Ok(commands) = read_and_parse_gerber(path.to_str().unwrap()) {
            match build_silkscreen_mesh(&commands, false, Some(thickness)) {
                Ok(mesh) => {
                    println!(
                        "Bottom silkscreen mesh created with {} vertices and {} faces",
                        mesh.vertices.len(),
                        mesh.faces.len()
                    );
                    pcb_model.meshes.push(mesh);
                }
                Err(e) => println!("Warning: Failed to create bottom silkscreen mesh: {}", e),
            }
        }
    }

    Ok(pcb_model)
}

/// Reads a Gerber file and parses its content into commands.
///
/// # Arguments
///
/// * `file_path` - Path to the Gerber file
///
/// # Returns
///
/// * `Result<Vec<gerber::types::Command>, String>` - The parsed Gerber commands or an error message
pub fn read_and_parse_gerber(file_path: &str) -> Result<Vec<gerber::types::Command>, String> {
    // Convert to absolute path for better debugging
    let absolute_path = Path::new(file_path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(file_path).to_path_buf());
    println!("Reading file: {:?}", absolute_path);

    // Read the file content
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Error reading file {}: {}", file_path, e))?;

    println!("Successfully read Gerber file: {}", file_path);

    // Parse the Gerber content
    let commands = gerber::parse::parse_gerber(&content)?;
    println!("Successfully parsed {} Gerber commands", commands.len());

    Ok(commands)
}

/// Creates a 3D mesh representing the PCB outline from the Edge Cuts layer.
///
/// This function:
/// 1. Extracts 2D outline points from Gerber commands
/// 2. Handles linear segments and arc segments
/// 3. Extrudes the 2D outline into a 3D mesh with proper thickness
///
/// # Arguments
///
/// * `commands` - The parsed Gerber commands from the Edge Cuts layer
/// * `thickness` - PCB thickness in mm (optional, defaults to 1.6mm)
///
/// # Returns
///
/// * `Result<Mesh, String>` - A 3D mesh representing the PCB board outline
pub fn build_edge_cuts_mesh(
    commands: &[gerber::types::Command],
    thickness: Option<f64>,
) -> Result<Mesh, String> {
    use gerber::types::{Command, InterpolationMode, Point};
    use intermediate::model::{Face, Point3D, Vertex};

    // PCB parameters
    let pcb_thickness = thickness.unwrap_or(1.6); // Use provided thickness or default to 1.6mm
    const POINTS_PER_ARC: usize = 16; // Number of points to use when approximating arcs

    // Collect 2D outline points from the Gerber commands
    let mut outline_points: Vec<Point> = Vec::new();
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut current_mode = InterpolationMode::Linear;
    let mut start_point: Option<Point> = None;

    // First pass: collect all points from the edge cuts outline
    for cmd in commands {
        match cmd {
            Command::Move { point } => {
                current_x = point.x;
                current_y = point.y;

                // If this is the first point, record it as the start point
                if start_point.is_none() {
                    start_point = Some(Point {
                        x: current_x,
                        y: current_y,
                    });
                }

                // Add the point to our outline
                outline_points.push(Point {
                    x: current_x,
                    y: current_y,
                });
            }
            Command::Draw { point } => {
                current_x = point.x;
                current_y = point.y;
                outline_points.push(Point {
                    x: current_x,
                    y: current_y,
                });
            }
            Command::ArcDraw {
                end_point,
                center_offset,
            } => {
                // For arcs, we need to generate points along the arc path
                let start_x = current_x;
                let start_y = current_y;
                let end_x = end_point.x;
                let end_y = end_point.y;
                let center_x = start_x + center_offset.x;
                let center_y = start_y + center_offset.y;

                // Calculate start and end angles
                let start_angle = (start_y - center_y).atan2(start_x - center_x);
                let end_angle = (end_y - center_y).atan2(end_x - center_x);

                // Calculate radius
                let radius = ((start_x - center_x).powi(2) + (start_y - center_y).powi(2)).sqrt();

                // Generate points along the arc
                let mut angle_diff = end_angle - start_angle;

                // Adjust angle difference based on the interpolation mode
                match current_mode {
                    InterpolationMode::ClockwiseCircular => {
                        if angle_diff > 0.0 {
                            angle_diff -= 2.0 * std::f64::consts::PI;
                        }
                    }
                    InterpolationMode::CounterClockwiseCircular => {
                        if angle_diff < 0.0 {
                            angle_diff += 2.0 * std::f64::consts::PI;
                        }
                    }
                    _ => {}
                }

                // Generate points along the arc
                for i in 1..=POINTS_PER_ARC {
                    let angle = start_angle + angle_diff * (i as f64 / POINTS_PER_ARC as f64);
                    let x = center_x + radius * angle.cos();
                    let y = center_y + radius * angle.sin();
                    outline_points.push(Point { x, y });
                }

                // Update current position
                current_x = end_x;
                current_y = end_y;
            }
            Command::SetInterpolationMode(mode) => {
                current_mode = mode.clone();
            }
            _ => {} // Ignore other commands
        }
    }

    // Make sure the outline is closed
    if let Some(start) = start_point {
        if let Some(last) = outline_points.last() {
            if (start.x != last.x) || (start.y != last.y) {
                outline_points.push(start);
            }
        }
    }

    // Convert 2D outline to 3D mesh by extruding
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    // Check if we have enough points
    if outline_points.len() < 3 {
        return Err("Not enough points to create a valid mesh".to_string());
    }

    // Create top and bottom vertices
    for point in &outline_points {
        // Top vertex
        vertices.push(Vertex {
            position: Point3D {
                x: point.x,
                y: point.y,
                z: pcb_thickness,
            },
            normal: Point3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        });

        // Bottom vertex
        vertices.push(Vertex {
            position: Point3D {
                x: point.x,
                y: point.y,
                z: 0.0,
            },
            normal: Point3D {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
        });
    }

    // Create top face (simple triangle fan)
    let num_points = outline_points.len();
    let mut top_face = Face {
        vertices: Vec::new(),
    };

    for i in 0..num_points {
        top_face.vertices.push(i * 2); // Even indices are top vertices
    }
    faces.push(top_face);

    // Create bottom face (reversed)
    let mut bottom_face = Face {
        vertices: Vec::new(),
    };

    for i in (0..num_points).rev() {
        bottom_face.vertices.push(i * 2 + 1); // Odd indices are bottom vertices
    }
    faces.push(bottom_face);

    // Create side faces (quads connecting top and bottom)
    for i in 0..num_points {
        let next_i = (i + 1) % num_points;

        let quad = Face {
            vertices: vec![
                i * 2,          // Current top
                i * 2 + 1,      // Current bottom
                next_i * 2 + 1, // Next bottom
                next_i * 2,     // Next top
            ],
        };

        faces.push(quad);
    }

    let mesh = Mesh {
        vertices,
        faces,
        layer_type: LayerType::EdgeCuts,
    };

    println!(
        "Edge cuts mesh created with {} vertices and {} faces",
        mesh.vertices.len(),
        mesh.faces.len()
    );

    Ok(mesh)
}

/// Creates a 3D mesh representing a copper layer (top or bottom).
///
/// Currently implements a placeholder visualization, but in a complete implementation
/// would trace copper tracks and pads with proper thickness.
///
/// # Arguments
///
/// * `_commands` - The parsed Gerber commands from the copper layer
/// * `is_top` - Whether this is the top copper layer (`true`) or bottom (`false`)
/// * `thickness` - PCB thickness in mm (optional, defaults to 1.6mm)
///
/// # Returns
///
/// * `Result<Mesh, String>` - A 3D mesh representing the copper layer
pub fn build_copper_mesh(
    _commands: &[gerber::types::Command],
    is_top: bool,
    thickness: Option<f64>,
) -> Result<Mesh, String> {
    use intermediate::model::{Face, Point3D, Vertex};

    // This is a placeholder implementation for copper layers
    // For a complete implementation, we would trace the copper traces and pads

    // Constants
    let pcb_thickness = thickness.unwrap_or(1.6); // Use provided thickness or default to 1.6mm
    #[allow(dead_code)]
    const COPPER_THICKNESS: f64 = 0.035; // Standard copper thickness in mm

    // For now, just create a simple rectangular placeholder mesh for copper
    let z_position = if is_top { pcb_thickness } else { 0.0 };

    // Create a small rectangle to represent copper
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    // Dummy vertices for a small copper square near the center
    let size = 10.0;
    let center_x = 150.0;
    let center_y = -90.0;

    // Add 4 corners of the square
    vertices.push(Vertex {
        position: Point3D {
            x: center_x - size,
            y: center_y - size,
            z: z_position,
        },
        normal: Point3D {
            x: 0.0,
            y: 0.0,
            z: if is_top { 1.0 } else { -1.0 },
        },
    });

    vertices.push(Vertex {
        position: Point3D {
            x: center_x + size,
            y: center_y - size,
            z: z_position,
        },
        normal: Point3D {
            x: 0.0,
            y: 0.0,
            z: if is_top { 1.0 } else { -1.0 },
        },
    });

    vertices.push(Vertex {
        position: Point3D {
            x: center_x + size,
            y: center_y + size,
            z: z_position,
        },
        normal: Point3D {
            x: 0.0,
            y: 0.0,
            z: if is_top { 1.0 } else { -1.0 },
        },
    });

    vertices.push(Vertex {
        position: Point3D {
            x: center_x - size,
            y: center_y + size,
            z: z_position,
        },
        normal: Point3D {
            x: 0.0,
            y: 0.0,
            z: if is_top { 1.0 } else { -1.0 },
        },
    });

    // Add a face with the 4 vertices
    faces.push(Face {
        vertices: vec![0, 1, 2, 3],
    });

    let mesh = Mesh {
        vertices,
        faces,
        layer_type: LayerType::Copper,
    };

    Ok(mesh)
}

/// Creates a 3D mesh representing a silkscreen layer (top or bottom).
///
/// Currently implements a placeholder visualization, but in a complete implementation
/// would trace silkscreen text and symbols with proper height.
///
/// # Arguments
///
/// * `_commands` - The parsed Gerber commands from the silkscreen layer
/// * `is_top` - Whether this is the top silkscreen layer (`true`) or bottom (`false`)
/// * `thickness` - PCB thickness in mm (optional, defaults to 1.6mm)
///
/// # Returns
///
/// * `Result<Mesh, String>` - A 3D mesh representing the silkscreen layer
pub fn build_silkscreen_mesh(
    _commands: &[gerber::types::Command],
    is_top: bool,
    thickness: Option<f64>,
) -> Result<Mesh, String> {
    use intermediate::model::{Face, Point3D, Vertex};

    // This is a placeholder implementation for silkscreen layers
    // For a complete implementation, we would trace the silkscreen text and symbols

    // Constants
    let pcb_thickness = thickness.unwrap_or(1.6); // Use provided thickness or default to 1.6mm
    const SILKSCREEN_THICKNESS: f64 = 0.01; // Standard silkscreen thickness in mm

    // For now, just create a simple rectangular placeholder mesh for silkscreen
    let z_position = if is_top {
        pcb_thickness + SILKSCREEN_THICKNESS
    } else {
        -SILKSCREEN_THICKNESS
    };

    // Create a small rectangle to represent silkscreen
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    // Dummy vertices for a small silkscreen square
    let size = 5.0;
    let center_x = 200.0;
    let center_y = -90.0;

    // Add 4 corners of the square
    vertices.push(Vertex {
        position: Point3D {
            x: center_x - size,
            y: center_y - size,
            z: z_position,
        },
        normal: Point3D {
            x: 0.0,
            y: 0.0,
            z: if is_top { 1.0 } else { -1.0 },
        },
    });

    vertices.push(Vertex {
        position: Point3D {
            x: center_x + size,
            y: center_y - size,
            z: z_position,
        },
        normal: Point3D {
            x: 0.0,
            y: 0.0,
            z: if is_top { 1.0 } else { -1.0 },
        },
    });

    vertices.push(Vertex {
        position: Point3D {
            x: center_x + size,
            y: center_y + size,
            z: z_position,
        },
        normal: Point3D {
            x: 0.0,
            y: 0.0,
            z: if is_top { 1.0 } else { -1.0 },
        },
    });

    vertices.push(Vertex {
        position: Point3D {
            x: center_x - size,
            y: center_y + size,
            z: z_position,
        },
        normal: Point3D {
            x: 0.0,
            y: 0.0,
            z: if is_top { 1.0 } else { -1.0 },
        },
    });

    // Add a face with the 4 vertices
    faces.push(Face {
        vertices: vec![0, 1, 2, 3],
    });

    let mesh = Mesh {
        vertices,
        faces,
        layer_type: LayerType::Silkscreen,
    };

    Ok(mesh)
}

/// Helper function to identify the likely layer type based on file name
pub fn identify_layer_type(file_path: &Path) -> LayerType {
    let file_name = file_path.file_name().unwrap().to_string_lossy().to_lowercase();
    
    if file_name.contains("edge") || file_name.contains("outline") || file_name.contains("cuts") {
        LayerType::EdgeCuts
    } else if file_name.contains("f.cu") || file_name.contains("f_cu") || file_name.contains("top.cu") {
        LayerType::Copper
    } else if file_name.contains("b.cu") || file_name.contains("b_cu") || file_name.contains("bottom.cu") {
        LayerType::Copper
    } else if file_name.contains("f.silk") || file_name.contains("f_silk") || file_name.contains("top.silk") {
        LayerType::Silkscreen
    } else if file_name.contains("b.silk") || file_name.contains("b_silk") || file_name.contains("bottom.silk") {
        LayerType::Silkscreen
    } else {
        LayerType::EdgeCuts // Default
    }
}

/// Helper function to analyze Gerber commands and return statistics
pub fn analyze_gerber_commands(commands: &[gerber::types::Command]) -> (usize, usize, usize, usize) {
    use gerber::types::Command;
    
    // Count different command types
    let mut move_count = 0;
    let mut draw_count = 0;
    let mut arc_count = 0;
    let mut other_count = 0;
    
    for cmd in commands {
        match cmd {
            Command::Move { .. } => move_count += 1,
            Command::Draw { .. } => draw_count += 1,
            Command::ArcDraw { .. } => arc_count += 1,
            _ => other_count += 1,
        }
    }
    
    (move_count, draw_count, arc_count, other_count)
}

/// Helper function to open a file with the system's default application
pub fn open_file(file_path: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(["/C", "start", "", file_path])
            .spawn()
            .map_err(|e| eprintln!("Failed to open file: {}", e))
            .ok();
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(file_path)
            .spawn()
            .map_err(|e| eprintln!("Failed to open file: {}", e))
            .ok();
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(file_path)
            .spawn()
            .map_err(|e| eprintln!("Failed to open file: {}", e))
            .ok();
    }
}