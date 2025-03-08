# pcbgen Development Guide

## Build Commands
- Build project: `cargo build`
- Run application: `cargo run -- convert --input <GERBER_DIRECTORY>`
- Run with options: `cargo run -- convert --input <DIR> --output <NAME> --format <obj|usdz|stl> --thickness <MM> --colors`
- Analyze Gerber files: `cargo run -- info --input <GERBER_DIRECTORY> --detailed`
- Run tests: `cargo test`
- Run specific test: `cargo test test_name`
- Generate documentation: `cargo doc --no-deps --open`

## CLI Usage
```
pcbgen [OPTIONS] <SUBCOMMAND>

Options:
  -v, --verbose               Increase verbosity (can be used multiple times)
  -q, --quiet                 Suppress all non-error output
  -h, --help                  Print help information
  -V, --version               Print version information

Subcommands:
  convert                     Convert Gerber files to 3D models
  info                        Display information about Gerber files
  help                        Print this message or the help of the given subcommand(s)

Convert Options:
  -i, --input <DIR>           Directory containing Gerber files (required)
  -o, --output <FILE>         Output file path without extension (default: output/pcb_model)
  -f, --format <FORMAT>       Export format: obj, usdz, or stl (default: obj)
  -t, --thickness <VALUE>     PCB thickness in mm (default: 1.6)
  -c, --colors                Enable colored visualization
  -p, --preview               Automatically open the model after creation

Info Options:
  -i, --input <PATH>          Directory or file to analyze (required)
  -d, --detailed              Show detailed layer information
```

## Code Style Guidelines
- **Formatting**: Follow rustfmt conventions
- **Naming**: Use snake_case for functions/variables, CamelCase for types/structs
- **Imports**: Group imports by module, standard library first
- **Documentation**: Use rustdoc style (`///` for functions, `//!` for modules)
- **Error Handling**: Use `Result<T, String>` for operations that can fail
- **Types**: Prefer explicit types for function parameters and returns
- **Structure**: Organize code into modules (gerber, intermediate, usdz)
- **Parsing**: Use nom combinators for parsing structured data
- **State**: Minimize mutable state, pass changes through function returns when possible
- **Intermediate Format**: Convert Gerber data to 3D model before exporting

## Project Architecture
The converter follows a pipeline: 
1. Scan directory for Gerber files by layer type
2. Parse Gerber files into structured commands using nom
3. Convert to intermediate 3D model with vertices, faces per layer
4. Export to 3D model format (OBJ, USDZ, or STL)

## Directory Structure
- `/src` - Source code
  - `/src/gerber` - Gerber file parsing
  - `/src/intermediate` - 3D model representation
  - `/src/usdz` - Export functionality
- `/examples` - Example code for using the library
- `/tests` - Integration tests
- `/output` - Generated 3D models
- `/gerbers` - Example Gerber files for testing