//! Data structures for the intermediate 3D model representation.

/// A 3D point with x, y, and z coordinates.
#[derive(Debug, Clone)]
pub struct Point3D {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Z coordinate (height)
    pub z: f64,
}

/// A vertex in 3D space with position and normal.
#[derive(Debug, Clone)]
pub struct Vertex {
    /// Position of the vertex in 3D space
    pub position: Point3D,
    /// Normal vector at this vertex
    pub normal: Point3D,
}

/// A face (polygon) in a 3D mesh, defined by indices into a vertex list.
#[derive(Debug, Clone)]
pub struct Face {
    /// Indices into a vertex list that form this face
    pub vertices: Vec<usize>, 
}

/// A 3D mesh representing a PCB layer, composed of vertices and faces.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// List of vertices in the mesh
    pub vertices: Vec<Vertex>,
    /// List of faces in the mesh
    pub faces: Vec<Face>,
    /// Type of PCB layer this mesh represents
    pub layer_type: LayerType,
}

/// Enumeration of PCB layer types.
#[derive(Debug, Clone, PartialEq)]
pub enum LayerType {
    /// Copper layer (traces and pads)
    Copper,
    /// Silkscreen layer (text and symbols)
    Silkscreen,
    /// Soldermask layer (green or other color coating)
    #[allow(dead_code)]
    Soldermask,
    /// Solder paste layer (for SMD components)
    #[allow(dead_code)]
    Paste,
    /// Board outline (edge cuts)
    EdgeCuts,
    /// Drill holes layer
    #[allow(dead_code)]
    Drill,
}

/// A complete PCB model composed of multiple layer meshes.
#[derive(Debug)]
pub struct PCBModel {
    /// List of layer meshes that make up the PCB
    pub meshes: Vec<Mesh>,
    /// Units used for coordinates (mm or inches)
    #[allow(dead_code)]
    pub units: Units,
}

/// Units of measurement for PCB coordinates.
#[derive(Debug, Clone, PartialEq)]
pub enum Units {
    /// Millimeters (most common)
    Millimeters,
    /// Inches (used in some older designs)
    #[allow(dead_code)]
    Inches,
}

/// Convert from Gerber 2D coordinates to 3D space.
///
/// This implementation sets the Z coordinate to 0.0 by default,
/// but it will typically be adjusted based on the layer.
impl From<crate::gerber::types::Point> for Point3D {
    fn from(point: crate::gerber::types::Point) -> Self {
        Point3D {
            x: point.x,
            y: point.y,
            z: 0.0, // Default to z=0, will be adjusted based on layer
        }
    }
}