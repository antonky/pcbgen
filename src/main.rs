//! # Gerber to USDZ CLI
//!
//! Command-line interface for the Gerber to USDZ converter.

use clap::{Parser, Subcommand, ValueEnum};
use pcbgen::{analyze_gerber_commands, identify_layer_type, open_file, process_gerber_files};
use std::path::Path;

/// pcbgen - Turn flat PCB files into beautiful 3D models
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Set verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all non-error output
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Transform Gerber files into interactive 3D models
    Convert {
        /// Directory containing Gerber files
        #[arg(short, long)]
        input: String,

        /// Output file path (without extension)
        #[arg(short, long, default_value = "output/pcb_model")]
        output: String,

        /// Export format (obj, usdz, etc.)
        #[arg(short, long, value_enum, default_value_t = Format::Obj)]
        format: Format,

        /// PCB thickness in mm
        #[arg(short, long, default_value_t = 1.6)]
        thickness: f64,

        /// Enable colored visualization
        #[arg(short, long)]
        colors: bool,

        /// Automatically open the model after creation
        #[arg(short, long)]
        preview: bool,
    },

    /// Inspect and analyze Gerber files without conversion
    Info {
        /// Directory or file to analyze
        #[arg(short, long)]
        input: String,

        /// Show detailed layer information
        #[arg(short, long)]
        detailed: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
enum Format {
    /// Wavefront OBJ format - Most compatible, supports colors
    Obj,
    /// Apple USDZ format - For AR/VR experiences on iOS devices
    Usdz,
    /// STL format - Industry standard for 3D printing and CAD
    Stl,
}

/// Main entry point for the application.
///
/// Processes command-line arguments and executes the appropriate subcommand.
fn main() {
    let cli = Cli::parse();

    // Setup logging based on verbosity
    let log_level = match (cli.quiet, cli.verbose) {
        (true, _) => 0,  // Quiet mode - only errors
        (false, 0) => 1, // Default - info
        (false, 1) => 2, // Verbose - debug
        (false, _) => 3, // Very verbose - trace
    };

    if !cli.quiet {
        println!(r#"
  _____   _____ ____   _____  ______ _   _ 
 |  __ \ / ____|  _ \ / ____|  ____| \ | |
 | |__) | |    | |_) | |  __| |__  |  \| |
 |  ___/| |    |  _ <| | |_ |  __| | . ` |
 | |    | |____| |_) | |__| | |____| |\  |
 |_|     \_____|____/ \_____|______|_| \_|
                                          
 Turn flat PCB designs into beautiful 3D models
 Version: {} | Made with Rust
"#, env!("CARGO_PKG_VERSION"));
    }

    // Ensure output directory exists
    std::fs::create_dir_all("output").unwrap_or_else(|e| {
        eprintln!("Warning: Failed to create output directory: {}", e);
    });

    // Execute the appropriate subcommand (or default to Convert)
    match cli.command.unwrap_or(Commands::Convert {
        input: String::from("."),
        output: String::from("output/pcb_model"),
        format: Format::Obj,
        thickness: 1.6,
        colors: false,
        preview: false,
    }) {
        Commands::Convert {
            input,
            output,
            format,
            thickness,
            colors,
            preview,
        } => {
            convert_command(
                &input, &output, format, thickness, colors, preview, log_level, cli.quiet,
            );
        }
        Commands::Info { input, detailed } => {
            info_command(&input, detailed, log_level, cli.quiet);
        }
    }
}

/// The convert subcommand - processes Gerber files and exports a 3D model
fn convert_command(
    input: &str,
    output: &str,
    format: Format,
    thickness: f64,
    colors: bool,
    preview: bool,
    log_level: u8,
    quiet: bool,
) {
    if log_level > 0 {
        println!("\nInput directory: {}", input);
        println!("Converting to: {}.{:?}", output, format);
        println!("PCB thickness: {}mm", thickness);
        
        if colors {
            println!("Color visualization enabled");
        }
        
        if preview {
            println!("Auto-preview enabled - will open after conversion");
        }
        
        println!("\nScanning for Gerber files...");
    }

    // Process Gerber files and build a 3D model
    let pcb_model = process_gerber_files(input, thickness).unwrap_or_else(|e| {
        eprintln!("\nError processing Gerber files: {}", e);
        eprintln!("Try using 'pcbgen info' to analyze your Gerber files before conversion.");
        std::process::exit(1);
    });

    // Print model info if not in quiet mode
    if log_level > 0 {
        println!("\nPCB Model created successfully with:");
        println!("   - {} mesh components", pcb_model.meshes.len());
        
        // Count different layer types
        let mut edge_cuts = 0;
        let mut copper = 0;
        let mut silkscreen = 0;
        
        for mesh in &pcb_model.meshes {
            match mesh.layer_type {
                pcbgen::intermediate::model::LayerType::EdgeCuts => edge_cuts += 1,
                pcbgen::intermediate::model::LayerType::Copper => copper += 1,
                pcbgen::intermediate::model::LayerType::Silkscreen => silkscreen += 1,
                _ => {}
            }
        }
        
        println!("   - {} Edge Cuts layer(s)", edge_cuts);
        println!("   - {} Copper layer(s)", copper);
        println!("   - {} Silkscreen layer(s)", silkscreen);
        
        println!("\nExporting model...");
    }

    // Export model in the requested format
    match format {
        Format::Obj => {
            let output_path = format!("{}.obj", output);
            match pcbgen::usdz::export::export_to_obj(&pcb_model, &output_path, colors) {
                Ok(_) => {
                    if !quiet {
                        println!("\nSuccessfully exported model to {}", output_path);
                        println!("   Format: OBJ with materials (.mtl)");
                        if colors {
                            println!("   Colors: Enabled (Edge Cuts=Green, Top Copper=Red, Bottom Copper=Blue)");
                        }
                    }

                    // Open the file if preview is requested
                    if preview {
                        if !quiet {
                            println!("Opening model in default viewer...");
                        }
                        open_file(&output_path);
                    }
                }
                Err(e) => eprintln!("Error exporting to OBJ: {}", e),
            }
        }
        Format::Usdz => {
            let output_path = format!("{}.usdz", output);
            match pcbgen::usdz::export::export_to_usdz(&pcb_model, &output_path) {
                Ok(_) => {
                    if !quiet {
                        println!("\nSuccessfully exported model to {}", output_path);
                        println!("   Format: USDZ (Apple AR/VR format)");
                        println!("   Note: View using AR Quick Look on iOS/iPadOS or macOS");
                    }

                    // Open the file if preview is requested
                    if preview {
                        if !quiet {
                            println!("Opening model in default viewer...");
                        }
                        open_file(&output_path);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Format::Stl => {
            eprintln!("STL export format not yet implemented");
            eprintln!("Please use OBJ or USDZ format for now. STL support coming soon!");
            std::process::exit(1);
        }
    }
}

/// The info subcommand - analyzes Gerber files and displays information
fn info_command(input: &str, detailed: bool, log_level: u8, _quiet: bool) {
    let input_path = Path::new(input);

    if !input_path.exists() {
        eprintln!("Input path does not exist: {}", input);
        std::process::exit(1);
    }

    if input_path.is_file() {
        // Analyze a single Gerber file
        println!("\nAnalyzing Gerber file: {}", input);

        match std::fs::read_to_string(input) {
            Ok(content) => match pcbgen::gerber::parse::parse_gerber(&content) {
                Ok(commands) => {
                    println!("  Valid Gerber file with {} commands", commands.len());

                    if detailed {
                        let (move_count, draw_count, arc_count, other_count) =
                            analyze_gerber_commands(&commands);

                        println!("    Command statistics:");
                        println!("      Move commands: {}", move_count);
                        println!("      Draw commands: {}", draw_count);
                        println!("      Arc commands: {}", arc_count);
                        println!("      Other commands: {}", other_count);
                    }
                }
                Err(e) => println!("  Not a valid Gerber file: {}", e),
            },
            Err(e) => println!("  Error reading file: {}", e),
        }
    } else {
        // Analyze a directory of Gerber files
        println!("\nAnalyzing Gerber files in directory: {}", input);

        // Find and categorize Gerber files similar to process_gerber_files
        let entries = std::fs::read_dir(input_path).unwrap_or_else(|e| {
            eprintln!("Error reading directory: {}", e);
            std::process::exit(1);
        });

        let mut gerber_files = Vec::new();

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "gbr" || ext == "GBR" {
                        gerber_files.push(path);
                    }
                }
            }
        }

        if gerber_files.is_empty() {
            println!("No Gerber files found in directory");
        } else {
            println!("Found {} Gerber files:", gerber_files.len());

            for file in gerber_files {
                let file_name = file.file_name().unwrap().to_string_lossy();
                println!("  {}", file_name);

                if detailed {
                    // Analyze each file if detailed info is requested
                    match std::fs::read_to_string(&file) {
                        Ok(content) => {
                            match pcbgen::gerber::parse::parse_gerber(&content) {
                                Ok(commands) => {
                                    println!(
                                        "    Valid Gerber file with {} commands",
                                        commands.len()
                                    );

                                    // Try to identify layer type
                                    let layer_type = identify_layer_type(&file);
                                    println!("    Likely layer type: {:?}", layer_type);

                                    if log_level > 1 {
                                        let (move_count, draw_count, arc_count, other_count) =
                                            analyze_gerber_commands(&commands);

                                        println!("    Command statistics:");
                                        println!("      Move commands: {}", move_count);
                                        println!("      Draw commands: {}", draw_count);
                                        println!("      Arc commands: {}", arc_count);
                                        println!("      Other commands: {}", other_count);
                                    }
                                }
                                Err(e) => println!("    Not a valid Gerber file: {}", e),
                            }
                        }
                        Err(e) => println!("    Error reading file: {}", e),
                    }
                }
            }
        }
    }
}
