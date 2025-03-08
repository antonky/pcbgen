//! Simple example of using the pcbgen library to convert Gerber files to OBJ

use pcbgen::{process_gerber_files, usdz::export::export_to_obj};
use std::path::Path;

fn main() -> Result<(), String> {
    println!("pcbgen - Simple Conversion Example");
    
    // Replace with the path to your Gerber files
    let input_dir = "gerbers";
    
    // Check if the input directory exists
    if !Path::new(input_dir).exists() {
        return Err(format!("Input directory not found: {}", input_dir));
    }
    
    // Process Gerber files with default thickness
    let pcb_model = process_gerber_files(input_dir, 1.6)?;
    
    println!("PCB model created with {} meshes", pcb_model.meshes.len());
    
    // Export to OBJ with colors enabled
    let output_path = "output/example_model.obj";
    export_to_obj(&pcb_model, output_path, true)?;
    
    println!("Successfully exported model to {}", output_path);
    
    Ok(())
}