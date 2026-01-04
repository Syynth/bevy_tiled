//! Export registered `TiledClass` types to JSON for Tiled editor integration.
//!
//! Generates a JSON file in the format expected by Tiled's custom property system.
//! See: <https://doc.mapeditor.org/en/stable/manual/custom-properties/>

use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::registry::TiledClassRegistry;

/// Export all registered `TiledClass` types to a JSON file for Tiled editor.
///
/// The generated JSON file defines custom class types that will appear in Tiled's
/// property editor dropdowns. This allows users to easily add custom components
/// to their maps, objects, and layers.
///
/// # Arguments
///
/// * `registry` - The `TiledClassRegistry` containing all registered types
/// * `output_path` - Path where the JSON file should be written
///
/// # Returns
///
/// `Ok(())` if export succeeded, `Err` if file writing failed
///
/// # Example
///
/// ```ignore
/// let registry = TiledClassRegistry::build();
/// export_types_to_json(&registry, "assets/tiled_types.json")?;
/// ```
///
/// # Note
///
/// The exact JSON format will be implemented in Phase 2 after examining
/// `bevy_ecs_tiled`'s export format. For Phase 1, this is a stub implementation.
pub fn export_types_to_json(
    registry: &TiledClassRegistry,
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let path = output_path.as_ref();

    // TODO: Phase 2 - Implement proper JSON schema generation
    // For now, just create a placeholder file to verify the infrastructure works

    let mut file = File::create(path)?;

    writeln!(file, "{{")?;
    writeln!(file, "  \"version\": \"0.1.0\",")?;
    writeln!(file, "  \"types\": [")?;

    let type_names: Vec<_> = registry.type_names().collect();
    for (i, name) in type_names.iter().enumerate() {
        let comma = if i < type_names.len() - 1 { "," } else { "" };
        writeln!(file, "    {{\"name\": \"{}\"}}{}", name, comma)?;
    }

    writeln!(file, "  ]")?;
    writeln!(file, "}}")?;

    Ok(())
}
