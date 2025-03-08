//! Export functionality for the 3D PCB model.
//!
//! Provides functions to export the PCB model to various formats.

use crate::intermediate::model::PCBModel;
/// Exports a PCB model to USDZ format.
///
/// This is a placeholder for future implementation.
///
/// # Arguments
///
/// * `_model` - The PCB model to export
/// * `_output_path` - Path where the USDZ file will be written
///
/// # Returns
///
/// * `Result<(), String>` - Success or error message
#[allow(dead_code)]
pub fn export_to_usdz(_model: &PCBModel, _output_path: &str) -> Result<(), String> {
    // This is a placeholder that will be implemented later
    Err("USDZ export not yet implemented".to_string())
}

/// Exports a PCB model to OBJ format.
///
/// Creates a Wavefront OBJ file that can be viewed in any 3D modeling software.
/// This is useful for debugging and visualization purposes.
///
/// # Arguments
///
/// * `model` - The PCB model to export
/// * `output_path` - Path where the OBJ file will be written
/// * `colors` - Whether to include color information and annotations
///
/// # Returns
///
/// * `Result<(), String>` - Success or error message
pub fn export_to_obj(model: &PCBModel, output_path: &str, colors: bool) -> Result<(), String> {
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::Path;
    use crate::intermediate::model::LayerType;
    
    let file = File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);
    
    writeln!(writer, "# PCB Model exported from Gerber").map_err(|e| format!("Write error: {}", e))?;
    
    // If colors mode is enabled, create material info
    if colors {
        // Get the base filename without extension
        let path = Path::new(output_path);
        let stem = path.file_stem().unwrap().to_string_lossy();
        let mtl_filename = format!("{}.mtl", stem);
        let mtl_path = path.with_file_name(&mtl_filename);
        
        // Add material library reference
        writeln!(writer, "mtllib {}", mtl_filename).map_err(|e| format!("Write error: {}", e))?;
        
        // Create the MTL file for materials
        let mtl_file = File::create(mtl_path).map_err(|e| format!("Failed to create MTL file: {}", e))?;
        let mut mtl_writer = BufWriter::new(mtl_file);
        
        // Write material definitions for each layer type
        writeln!(mtl_writer, "# Layer materials for debugging").map_err(|e| format!("Write error: {}", e))?;
        
        // Edge Cuts - Green
        writeln!(mtl_writer, "newmtl EdgeCuts").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ka 0.0 0.5 0.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Kd 0.0 0.8 0.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ks 0.1 0.1 0.1").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "d 1.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "illum 2").map_err(|e| format!("Write error: {}", e))?;
        
        // Top Copper - Red
        writeln!(mtl_writer, "newmtl TopCopper").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ka 0.5 0.0 0.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Kd 0.8 0.0 0.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ks 0.8 0.8 0.8").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "d 1.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "illum 2").map_err(|e| format!("Write error: {}", e))?;
        
        // Bottom Copper - Blue
        writeln!(mtl_writer, "newmtl BottomCopper").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ka 0.0 0.0 0.5").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Kd 0.0 0.0 0.8").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ks 0.8 0.8 0.8").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "d 1.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "illum 2").map_err(|e| format!("Write error: {}", e))?;
        
        // Top Silkscreen - White
        writeln!(mtl_writer, "newmtl TopSilkscreen").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ka 0.9 0.9 0.9").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Kd 1.0 1.0 1.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ks 0.0 0.0 0.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "d 1.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "illum 2").map_err(|e| format!("Write error: {}", e))?;
        
        // Bottom Silkscreen - Yellow
        writeln!(mtl_writer, "newmtl BottomSilkscreen").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ka 0.5 0.5 0.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Kd 0.8 0.8 0.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "Ks 0.0 0.0 0.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "d 1.0").map_err(|e| format!("Write error: {}", e))?;
        writeln!(mtl_writer, "illum 2").map_err(|e| format!("Write error: {}", e))?;
    }
    
    let mut global_vertex_index = 1; // OBJ indices start at 1
    
    for mesh in &model.meshes {
        writeln!(writer, "\n# Layer type: {:?}", mesh.layer_type).map_err(|e| format!("Write error: {}", e))?;
        
        // If in colors mode, assign material based on layer type
        if colors {
            let material = match mesh.layer_type {
                LayerType::EdgeCuts => "EdgeCuts",
                LayerType::Copper => {
                    // Determine if it's top or bottom based on vertices z position
                    if !mesh.vertices.is_empty() && mesh.vertices[0].position.z > 0.5 {
                        "TopCopper"
                    } else {
                        "BottomCopper"
                    }
                },
                LayerType::Silkscreen => {
                    // Determine if it's top or bottom based on vertices z position
                    if !mesh.vertices.is_empty() && mesh.vertices[0].position.z > 0.5 {
                        "TopSilkscreen"
                    } else {
                        "BottomSilkscreen"
                    }
                },
                _ => "EdgeCuts" // Default for other layer types
            };
            
            writeln!(writer, "usemtl {}", material).map_err(|e| format!("Write error: {}", e))?;
            
            // Add debug annotation - a small text comment with layer info and vertex count
            writeln!(writer, "# DEBUG: Layer {:?} with {} vertices", 
                    mesh.layer_type, mesh.vertices.len())
                .map_err(|e| format!("Write error: {}", e))?;
        }
        
        // Write vertices
        for vertex in &mesh.vertices {
            writeln!(
                writer, 
                "v {} {} {}",
                vertex.position.x,
                vertex.position.y,
                vertex.position.z
            ).map_err(|e| format!("Write error: {}", e))?;
            
            // Write vertex normals
            writeln!(
                writer,
                "vn {} {} {}",
                vertex.normal.x,
                vertex.normal.y,
                vertex.normal.z
            ).map_err(|e| format!("Write error: {}", e))?;
        }
        
        // Write faces
        for face in &mesh.faces {
            if face.vertices.len() >= 3 {
                write!(writer, "f").map_err(|e| format!("Write error: {}", e))?;
                
                for &vertex_idx in &face.vertices {
                    let obj_idx = global_vertex_index + vertex_idx;
                    write!(writer, " {}//{}", obj_idx, obj_idx).map_err(|e| format!("Write error: {}", e))?;
                }
                
                writeln!(writer).map_err(|e| format!("Write error: {}", e))?;
            }
        }
        
        global_vertex_index += mesh.vertices.len();
    }
    
    Ok(())
}