//! Gerber file parser module.
//!
//! The Gerber format is the standard file format for PCB manufacturing.
//! This module provides functionality to parse Gerber files and extract
//! the command structures needed to build a 3D model.
//!
//! ## Submodules
//!
//! - `types`: Defines Gerber file structures and commands.
//! - `parse`: Implements the parser for Gerber files.

pub mod parse;
pub mod types;