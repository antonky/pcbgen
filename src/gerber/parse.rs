//! Parser implementation for Gerber files using nom.
//!
//! Provides functionality to parse Gerber files into structured commands.
//! 
//! This module uses the nom parsing library to implement a robust parser for
//! the Gerber file format. It breaks down the parsing process into specialized
//! functions for each type of command, making the code more maintainable and
//! easier to extend.
//!
//! ## Parsing strategy
//! 
//! The parser processes Gerber files line by line, applying different parsers
//! based on the command type:
//!
//! - Format specification (`%FSLAX...`)
//! - Units setting (`%MOMM*%` or `%MOIN*%`)
//! - Aperture definitions (`%ADD...`)
//! - Interpolation mode commands (`G01`, `G02`, `G03`)
//! - Region commands (`G36`, `G37`)
//! - Drawing commands (`D01`, `D02`, `D03`)
//! - Aperture selection commands (`D10*` etc.)
//!
//! The parser maintains state for current coordinates, format specification,
//! and interpolation mode, which is needed to properly interpret commands.

use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    combinator::value,
};

use crate::gerber::types::{Aperture, Command, InterpolationMode, Point};

/// Main parser function for Gerber files.
/// Parses a Gerber file's content into a list of structured commands.
///
/// # Arguments
///
/// * `content` - The content of the Gerber file as a string
///
/// # Returns
///
/// * `Result<Vec<Command>, String>` - The parsed commands on success, or an error message
pub fn parse_gerber(content: &str) -> Result<Vec<Command>, String> {
    // Context for parsing
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut integer_digits = 2;
    let mut decimal_digits = 4;
    let mut current_interpolation = InterpolationMode::Linear;
    
    // Results
    let mut commands = Vec::new();
    
    // Process each line
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("G04") {
            continue;
        }
        
        // Try to parse the line with different parsers
        if let Ok((_, cmd)) = parse_format_spec(line) {
            integer_digits = cmd.0;
            decimal_digits = cmd.1;
            commands.push(Command::FormatSpecification {
                integer_digits,
                decimal_digits,
            });
        } else if let Ok((_, Command::SetUnitsMM)) = parse_units_mm(line) {
            commands.push(Command::SetUnitsMM);
        } else if let Ok((_, Command::SetUnitsInch)) = parse_units_inch(line) {
            commands.push(Command::SetUnitsInch);
        } else if let Ok((_, aperture_def)) = parse_aperture_definition(line) {
            commands.push(aperture_def);
        } else if let Ok((_, mode)) = parse_interpolation_mode(line) {
            current_interpolation = mode.clone();
            commands.push(Command::SetInterpolationMode(mode));
        } else if let Ok((_, Command::BeginRegion)) = parse_begin_region(line) {
            commands.push(Command::BeginRegion);
        } else if let Ok((_, Command::EndRegion)) = parse_end_region(line) {
            commands.push(Command::EndRegion);
        } else if let Ok((_, Command::EndOfFile)) = parse_end_of_file(line) {
            commands.push(Command::EndOfFile);
        } else if let Ok((_, aperture_select)) = parse_aperture_selection(line) {
            commands.push(aperture_select);
        } else if let Some(cmd) = parse_draw_command(line, &mut current_x, &mut current_y, 
                                              integer_digits, decimal_digits, &current_interpolation) {
            commands.push(cmd);
        }
        // Other commands could be added here
    }
    
    Ok(commands)
}

/// Parse a format specification line like %FSLAX46Y46*%
///
/// This function extracts the format specification from Gerber files, which defines
/// how coordinate values should be interpreted. The format specifies the number of
/// integer digits and decimal digits.
///
/// # Example
///
/// `%FSLAX46Y46*%` specifies a format with 4 integer digits and 6 decimal digits.
///
/// # Returns
///
/// A tuple containing (integer_digits, decimal_digits) on success.
fn parse_format_spec(input: &str) -> IResult<&str, (u8, u8)> {
    // Extract the format part between %FSLAX and *%
    if let Some(format_str) = input.strip_prefix("%FSLAX") {
        if let Some(format_str) = format_str.strip_suffix("*%") {
            if let Some(pos) = format_str.find('Y') {
                let x_format = &format_str[..pos];
                
                // Get integer and decimal digits
                if x_format.len() == 2 {
                    if let (Some(int_digit), Some(dec_digit)) = (
                        x_format.chars().next().and_then(|c| c.to_digit(10)),
                        x_format.chars().nth(1).and_then(|c| c.to_digit(10))
                    ) {
                        return Ok(("", (int_digit as u8, dec_digit as u8)));
                    }
                }
            }
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

/// Parse units set to millimeters: %MOMM*%
///
/// Recognizes the command that sets the units to millimeters.
/// This affects how coordinates are interpreted.
fn parse_units_mm(input: &str) -> IResult<&str, Command> {
    value(Command::SetUnitsMM, tag("%MOMM*%"))(input)
}

/// Parse units set to inches: %MOIN*%
///
/// Recognizes the command that sets the units to inches.
/// This affects how coordinates are interpreted.
fn parse_units_inch(input: &str) -> IResult<&str, Command> {
    value(Command::SetUnitsInch, tag("%MOIN*%"))(input)
}

/// Parse aperture definition like %ADD10C,0.1*%
///
/// Apertures define the shape and size used for drawing operations.
/// This function handles circle and rectangle apertures:
///
/// - Circle: %ADD10C,0.1*% (aperture D10 is a circle with diameter 0.1)
/// - Rectangle: %ADD11R,0.1X0.2*% (aperture D11 is a rectangle 0.1×0.2)
///
/// More aperture types could be added in the future.
fn parse_aperture_definition(input: &str) -> IResult<&str, Command> {
    if let Some(aperture_def) = input.strip_prefix("%ADD") {
        if let Some(aperture_def) = aperture_def.strip_suffix("*%") {
            // First try to parse a circle aperture
            if let Some(pos) = aperture_def.find('C') {
                let code_str = &aperture_def[..pos];
                let params_str = &aperture_def[pos+1..];
                
                if let Ok(code) = code_str.parse::<u32>() {
                    if params_str.starts_with(',') {
                        let diameter_str = &params_str[1..];
                        if let Ok(diameter) = diameter_str.parse::<f64>() {
                            return Ok(("", Command::DefineAperture {
                                code,
                                aperture: Aperture::Circle { diameter },
                            }));
                        }
                    }
                }
            }
            
            // Then try to parse a rectangle aperture
            if let Some(pos) = aperture_def.find('R') {
                let code_str = &aperture_def[..pos];
                let params_str = &aperture_def[pos+1..];
                
                if let Ok(code) = code_str.parse::<u32>() {
                    if params_str.starts_with(',') {
                        let params = &params_str[1..];
                        if let Some(x_pos) = params.find('X') {
                            let width_str = &params[..x_pos];
                            let height_str = &params[x_pos+1..];
                            
                            if let (Ok(width), Ok(height)) = (width_str.parse::<f64>(), height_str.parse::<f64>()) {
                                return Ok(("", Command::DefineAperture {
                                    code,
                                    aperture: Aperture::Rectangle { width, height },
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

/// Parse interpolation mode: G01, G02, G03
///
/// The interpolation mode defines how drawing operations are performed:
///
/// - G01: Linear interpolation (straight lines)
/// - G02: Clockwise circular interpolation (clockwise arcs)
/// - G03: Counterclockwise circular interpolation (counterclockwise arcs)
fn parse_interpolation_mode(input: &str) -> IResult<&str, InterpolationMode> {
    alt((
        value(InterpolationMode::Linear, tag("G01")),
        value(InterpolationMode::ClockwiseCircular, tag("G02")),
        value(InterpolationMode::CounterClockwiseCircular, tag("G03")),
    ))(input)
}

/// Parse begin region command: G36
///
/// Recognizes the command that starts a region definition.
/// Regions are filled polygons in Gerber files.
fn parse_begin_region(input: &str) -> IResult<&str, Command> {
    value(Command::BeginRegion, tag("G36"))(input)
}

/// Parse end region command: G37
///
/// Recognizes the command that ends a region definition.
/// Must be paired with a preceding G36 command.
fn parse_end_region(input: &str) -> IResult<&str, Command> {
    value(Command::EndRegion, tag("G37"))(input)
}

/// Parse end of file command: M02
///
/// Recognizes the end of file marker in Gerber files.
/// This should be the last command in a properly formatted Gerber file.
fn parse_end_of_file(input: &str) -> IResult<&str, Command> {
    value(Command::EndOfFile, tag("M02"))(input)
}

/// Parse aperture selection like D10*
///
/// Aperture selection commands specify which aperture to use for
/// subsequent drawing operations. Aperture numbers 10 and higher
/// are used for custom apertures defined with %ADD commands.
fn parse_aperture_selection(input: &str) -> IResult<&str, Command> {
    if input.starts_with('D') && input.ends_with('*') {
        let code_str = &input[1..input.len()-1];
        if let Ok(code) = code_str.parse::<u32>() {
            if code >= 10 {  // D10 and above are aperture selections
                return Ok(("", Command::SelectAperture { code }));
            }
        }
    }
    
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
}

/// Parse a draw command (D01, D02, D03) with coordinates
///
/// Drawing commands are the core of Gerber files. They define operations like:
///
/// - D01: Draw line or arc to the specified coordinates
/// - D02: Move to the specified coordinates (without drawing)
/// - D03: Flash the current aperture at the specified coordinates
///
/// For arc drawing (when in G02/G03 mode), I/J parameters specify the
/// offset from the current point to the center of the arc.
///
/// This function extracts coordinates and updates the current position state.
fn parse_draw_command(
    input: &str,
    current_x: &mut f64,
    current_y: &mut f64,
    integer_digits: u8,
    decimal_digits: u8,
    current_interpolation: &InterpolationMode
) -> Option<Command> {
    // Check if it's a draw command
    if !input.ends_with('*') {
        return None;
    }
    
    // Extract coordinates
    let mut x = None;
    let mut y = None;
    let mut i = None;
    let mut j = None;
    
    // Parse X coordinate
    if let Some(x_pos) = input.find('X') {
        let x_end = find_next_letter(input, x_pos + 1);
        let x_str = &input[x_pos+1..x_end];
        if let Ok(val) = parse_coordinate(x_str, integer_digits, decimal_digits) {
            x = Some(val);
        }
    }
    
    // Parse Y coordinate
    if let Some(y_pos) = input.find('Y') {
        let y_end = find_next_letter(input, y_pos + 1);
        let y_str = &input[y_pos+1..y_end];
        if let Ok(val) = parse_coordinate(y_str, integer_digits, decimal_digits) {
            y = Some(val);
        }
    }
    
    // Parse I coordinate (for arcs)
    if let Some(i_pos) = input.find('I') {
        let i_end = find_next_letter(input, i_pos + 1);
        let i_str = &input[i_pos+1..i_end];
        if let Ok(val) = parse_coordinate(i_str, integer_digits, decimal_digits) {
            i = Some(val);
        }
    }
    
    // Parse J coordinate (for arcs)
    if let Some(j_pos) = input.find('J') {
        let j_end = find_next_letter(input, j_pos + 1);
        let j_str = &input[j_pos+1..j_end];
        if let Ok(val) = parse_coordinate(j_str, integer_digits, decimal_digits) {
            j = Some(val);
        }
    }
    
    // Update current position
    if let Some(x_val) = x {
        *current_x = x_val;
    }
    if let Some(y_val) = y {
        *current_y = y_val;
    }
    
    // Determine command type
    if input.contains("D01") || input.contains("D1") {
        // Draw command
        if matches!(current_interpolation, InterpolationMode::ClockwiseCircular | InterpolationMode::CounterClockwiseCircular) 
           && i.is_some() && j.is_some() {
            // Arc draw
            Some(Command::ArcDraw {
                end_point: Point { x: *current_x, y: *current_y },
                center_offset: Point { x: i.unwrap(), y: j.unwrap() },
            })
        } else {
            // Linear draw
            Some(Command::Draw {
                point: Point { x: *current_x, y: *current_y },
            })
        }
    } else if input.contains("D02") || input.contains("D2") {
        // Move command
        Some(Command::Move {
            point: Point { x: *current_x, y: *current_y },
        })
    } else if input.contains("D03") || input.contains("D3") {
        // Flash command
        Some(Command::Flash {
            point: Point { x: *current_x, y: *current_y },
        })
    } else {
        None
    }
}

/// Find the next letter or symbol in a string
/// 
/// Utility function used when parsing coordinates to find where
/// the current coordinate value ends and the next parameter begins.
///
/// # Arguments
///
/// * `input` - The string to search
/// * `start` - The position to start searching from
///
/// # Returns
///
/// The position of the next command letter or symbol, or the end of the string.
fn find_next_letter(input: &str, start: usize) -> usize {
    for (i, c) in input[start..].char_indices() {
        if c == 'X' || c == 'Y' || c == 'I' || c == 'J' || c == 'D' || c == '*' {
            return start + i;
        }
    }
    input.len()
}

/// Parse a coordinate value based on the Gerber format specification.
///
/// Handles Gerber coordinates with or without decimal points, applying the
/// specified format (number of integer and decimal digits).
///
/// # Arguments
///
/// * `coord_str` - The coordinate string to parse
/// * `integer_digits` - Number of digits before the decimal point
/// * `decimal_digits` - Number of digits after the decimal point
///
/// # Returns
///
/// * `Result<f64, String>` - The parsed coordinate value, or an error message
fn parse_coordinate(coord_str: &str, integer_digits: u8, decimal_digits: u8) -> Result<f64, String> {
    // For Gerber coordinates without a decimal point, we need to insert it based on format
    let val = if coord_str.contains('.') {
        // Already has decimal point
        coord_str.parse::<f64>().map_err(|_| format!("Invalid coordinate: {}", coord_str))?
    } else {
        // Need to insert decimal point based on format
        let total_digits = integer_digits + decimal_digits;
        let mut value_str = coord_str.to_string();
        
        // Handle negative sign
        let is_negative = value_str.starts_with('-');
        if is_negative {
            value_str = value_str[1..].to_string();
        }
        
        // Pad with leading zeros if needed
        while value_str.len() < total_digits as usize {
            value_str.insert(0, '0');
        }
        
        // Insert decimal point
        if decimal_digits > 0 {
            let decimal_pos = value_str.len() - decimal_digits as usize;
            value_str.insert(decimal_pos, '.');
        }
        
        // Restore negative sign if needed
        if is_negative {
            value_str.insert(0, '-');
        }
        
        value_str.parse::<f64>().map_err(|_| format!("Invalid coordinate: {}", coord_str))?
    };
    
    Ok(val)
}