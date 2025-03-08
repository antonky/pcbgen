# pcbgen

A Rust utility to convert PCB Gerber files to 3D models in USDZ, OBJ, and STL formats.

## Features

- Convert Gerber files to 3D models
- Support for multiple PCB layers:
  - Edge Cuts (board outline)
  - Copper layers (top and bottom)
  - Silkscreen layers (top and bottom)
- Automatic layer detection by filename
- Configurable PCB thickness
- Multiple export formats (OBJ, USDZ, STL)
- Color visualization for easier layer identification
- Automatic file preview option
- Gerber file analysis capabilities

## Installation

### Prerequisites

- Rust (cargo)

### Building from source

```bash
git clone https://github.com/yourusername/pcbgen.git
cd pcbgen
cargo build --release
```

The binary will be located at `target/release/pcbgen`.

## Usage

The tool has a command-line interface with subcommands:

```bash
# General format
pcbgen [OPTIONS] <SUBCOMMAND>

# Get help
pcbgen --help
```

### Global Options

- `-v, --verbose` - Increase verbosity (can be used multiple times)
- `-q, --quiet` - Suppress all non-error output
- `-h, --help` - Print help information
- `-V, --version` - Print version information

### Subcommands

#### Convert

Convert Gerber files to 3D models:

```bash
pcbgen convert --input <GERBER_DIRECTORY> [OPTIONS]
```

Options:
- `-i, --input <DIR>` - Directory containing Gerber files (required)
- `-o, --output <FILE>` - Output file path without extension (default: output/pcb_model)
- `-f, --format <FORMAT>` - Export format: obj, usdz, or stl (default: obj)
- `-t, --thickness <VALUE>` - PCB thickness in mm (default: 1.6)
- `-c, --colors` - Enable colored visualization
- `-p, --preview` - Automatically open the model after creation

#### Info

Analyze Gerber files without conversion:

```bash
pcbgen info --input <GERBER_DIRECTORY or FILE> [OPTIONS]
```

Options:
- `-i, --input <PATH>` - Directory or file to analyze (required)
- `-d, --detailed` - Show detailed layer information

### Color Visualization

When using the `--colors` flag, the tool adds helpful visualization features:

- Color-coded layers for easy identification:
  - Edge Cuts - Green
  - Top Copper - Red
  - Bottom Copper - Blue
  - Top Silkscreen - White
  - Bottom Silkscreen - Yellow
- Layer annotations in the OBJ file
- Material definitions (.mtl file) for realistic rendering

### Examples

```bash
# Convert Gerber files to OBJ with default settings
pcbgen convert --input gerbers

# Convert to USDZ with custom thickness, output name, and colors
pcbgen convert --input gerbers --format usdz --thickness 2.0 --output my_pcb --colors

# Convert and automatically open the result
pcbgen convert --input gerbers --preview

# Analyze Gerber files in a directory
pcbgen info --input gerbers

# Get detailed information about a single Gerber file
pcbgen info --input gerbers/reference-pcb-Edge_Cuts.gbr --detailed

# Quiet mode with only error output
pcbgen --quiet convert --input gerbers
```

## Gerber File Format

Gerber is the standard file format used for PCB manufacturing. The tool uses the following naming conventions to automatically identify layer types:

- Edge Cuts: Files containing "edge", "outline", or "cuts"
- Top Copper: Files containing "f.cu", "f_cu", or "top.cu"
- Bottom Copper: Files containing "b.cu", "b_cu", or "bottom.cu"
- Top Silkscreen: Files containing "f.silk", "f_silk", or "top.silk"
- Bottom Silkscreen: Files containing "b.silk", "b_silk", or "bottom.silk"

## License

[MIT](LICENSE)