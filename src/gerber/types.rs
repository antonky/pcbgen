//! Data structures for Gerber file representation.
//!
//! This module defines the types used to represent Gerber file contents,
//! including points, apertures, and commands.

/// A 2D point in Gerber coordinates.
#[derive(Debug, Clone)]
pub struct Point {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
}

/// Aperture definitions for Gerber files.
///
/// Apertures define shapes used for drawing in Gerber files.
#[derive(Debug, Clone)]
pub enum Aperture {
    /// Circular aperture with diameter
    #[allow(dead_code)]
    Circle { diameter: f64 },
    /// Rectangular aperture with width and height
    #[allow(dead_code)]
    Rectangle { width: f64, height: f64 },
    // More aperture types can be added later
}

/// Interpolation modes for drawing operations.
///
/// Defines how line segments or arcs are drawn.
#[derive(Debug, Clone)]
pub enum InterpolationMode {
    /// Straight line segments
    Linear,
    /// Clockwise arc segments
    ClockwiseCircular,
    /// Counter-clockwise arc segments
    CounterClockwiseCircular,
}

/// Gerber commands.
///
/// Represents the various commands found in Gerber files,
/// including format specifications, drawing operations, etc.
#[derive(Debug, Clone)]
pub enum Command {
    /// Format specification (eg. %FSLAX46Y46*%)
    #[allow(dead_code)]
    FormatSpecification {
        /// Number of digits before decimal point
        integer_digits: u8,
        /// Number of digits after decimal point
        decimal_digits: u8,
    },
    /// Set units to millimeters (%MOMM*%)
    SetUnitsMM,
    /// Set units to inches (%MOIN*%)
    SetUnitsInch,
    /// Set interpolation mode (G01, G02, G03)
    SetInterpolationMode(InterpolationMode),
    /// Move to a point without drawing (D02)
    Move { point: Point },
    /// Draw to a point (D01)
    Draw { point: Point },
    /// Flash aperture at a point (D03)
    #[allow(dead_code)]
    Flash { point: Point },
    /// Draw an arc to a point
    ArcDraw { 
        /// End point of the arc
        end_point: Point, 
        /// Center offset from current point (I and J values)
        center_offset: Point,
    },
    /// Select an aperture (D10, D11, etc)
    #[allow(dead_code)]
    SelectAperture { code: u32 },
    /// Define an aperture (%ADD10C,0.1*%)
    #[allow(dead_code)]
    DefineAperture { code: u32, aperture: Aperture },
    /// Begin a region (G36)
    BeginRegion,
    /// End a region (G37)
    EndRegion,
    /// End of file (M02)
    EndOfFile,
}

/// Complete Gerber file representation.
///
/// Contains all commands parsed from a Gerber file.
#[allow(dead_code)]
#[derive(Debug)]
pub struct GerberFile {
    /// List of commands in the Gerber file
    pub commands: Vec<Command>,
}