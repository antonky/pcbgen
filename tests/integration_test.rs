use pcbgen::{
    identify_layer_type,
    analyze_gerber_commands,
    gerber::parse,
};
use std::path::Path;

#[test]
fn test_layer_type_detection() {
    // Test edge cuts detection
    let edge_cuts = Path::new("test_edge_cuts.gbr");
    assert_eq!(
        format!("{:?}", identify_layer_type(edge_cuts)),
        "EdgeCuts"
    );
    
    // Test copper layers detection
    let top_copper = Path::new("test_f.cu.gbr");
    assert_eq!(
        format!("{:?}", identify_layer_type(top_copper)),
        "Copper"
    );
    
    // Test silkscreen layers detection
    let top_silk = Path::new("test_f_silk.gbr");
    assert_eq!(
        format!("{:?}", identify_layer_type(top_silk)),
        "Silkscreen"
    );
}

#[test]
fn test_gerber_command_parsing() {
    // Simple Gerber file content
    let content = r#"
G04 Gerber test file*
%FSLAX46Y46*%
%MOMM*%
G04 Set coordinate format to 4.6 mm*
G01*
G04 Linear interpolation mode*
D10*
G04 Select aperture 10*
X1000000Y1000000D02*
G04 Move to 10,10 mm*
X2000000Y1000000D01*
G04 Draw to 20,10 mm*
M02*
G04 End of file*
    "#;
    
    // Parse the content
    let commands = parse::parse_gerber(content).expect("Failed to parse Gerber content");
    
    // Analyze commands
    let (moves, draws, arcs, others) = analyze_gerber_commands(&commands);
    
    // Verify we have at least one move and one draw
    assert!(moves >= 1, "Should have at least 1 move command");
    assert!(draws >= 1, "Should have at least 1 draw command");
    assert_eq!(arcs, 0, "Should have no arc commands");
    assert!(others > 0, "Should have some other commands");
}