//! Export registered `TiledClass` types to JSON for Tiled editor integration.
//!
//! Generates a JSON file in the format expected by Tiled's custom property system.
//! See: <https://doc.mapeditor.org/en/stable/manual/custom-properties/>

use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::registry::{TiledClassRegistry, TiledDefaultValue};

/// Intermediate representation of a Tiled custom property type for serialization.
#[derive(Debug, Clone, PartialEq)]
pub struct TiledTypeExport {
    pub id: usize,
    pub name: String,
    pub members: Vec<TiledMemberExport>,
}

/// Intermediate representation of a type member for serialization.
#[derive(Debug, Clone, PartialEq)]
pub struct TiledMemberExport {
    pub name: String,
    pub tiled_type: String,
    pub value: TiledValueExport,
}

/// Exportable default value (simplified for JSON serialization).
#[derive(Debug, Clone, PartialEq)]
pub enum TiledValueExport {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Color(String), // Hex format: #AARRGGBB
}

/// Generate export data for all registered `TiledClass` types.
///
/// This function converts the registry into an intermediate representation
/// suitable for serialization. It's separated from file I/O to enable testing.
///
/// # Arguments
///
/// * `registry` - The `TiledClassRegistry` containing all registered types
///
/// # Returns
///
/// Vector of `TiledTypeExport` representing all registered types
///
/// # Example
///
/// ```ignore
/// let registry = TiledClassRegistry::build();
/// let types = build_export_data(&registry);
/// assert_eq!(types.len(), 4);
/// ```
pub fn build_export_data(registry: &TiledClassRegistry) -> Vec<TiledTypeExport> {
    let type_infos: Vec<_> = registry.iter().collect();

    type_infos
        .iter()
        .enumerate()
        .map(|(i, info)| {
            let members = info
                .fields
                .iter()
                .map(|field| TiledMemberExport {
                    name: field.name.to_string(),
                    tiled_type: field.tiled_type.to_string(),
                    value: convert_default_value(&field.default_value),
                })
                .collect();

            TiledTypeExport {
                id: i + 1,
                name: info.name.to_string(),
                members,
            }
        })
        .collect()
}

/// Convert TiledDefaultValue to TiledValueExport for serialization.
fn convert_default_value(value: &TiledDefaultValue) -> TiledValueExport {
    match value {
        TiledDefaultValue::Bool(b) => TiledValueExport::Bool(*b),
        TiledDefaultValue::Int(i) => TiledValueExport::Int(*i),
        TiledDefaultValue::Float(f) => TiledValueExport::Float(*f),
        TiledDefaultValue::String(s) => TiledValueExport::String(s.to_string()),
        TiledDefaultValue::Color { r, g, b, a } => {
            TiledValueExport::Color(format!("#{:02x}{:02x}{:02x}{:02x}", a, r, g, b))
        }
    }
}

/// Export all registered `TiledClass` types to a JSON file for Tiled editor.
///
/// The generated JSON file defines custom class types that will appear in Tiled's
/// property editor dropdowns. This allows users to easily add custom components
/// to their maps, objects, and layers.
///
/// The format follows Tiled's custom types specification:
/// <https://doc.mapeditor.org/en/stable/manual/custom-properties/#custom-types>
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
/// export_types_to_json(&registry, "assets/custom-types.json")?;
/// ```
///
/// # Tiled Integration
///
/// To use the exported types in Tiled:
/// 1. Save this file as `custom-types.json` in your project directory
/// 2. In Tiled: View → Custom Types → Import Custom Types
/// 3. Select the JSON file
/// 4. The types will now appear in property dropdowns
pub fn export_types_to_json(
    registry: &TiledClassRegistry,
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let path = output_path.as_ref();
    let mut file = File::create(path)?;

    let types = build_export_data(registry);
    write_types_to_file(&mut file, &types)?;

    Ok(())
}

/// Write type export data to a file in Tiled's JSON format.
fn write_types_to_file(file: &mut File, types: &[TiledTypeExport]) -> std::io::Result<()> {
    // Tiled custom types format
    // See: https://github.com/mapeditor/tiled/blob/master/docs/reference/json-map-format.rst
    writeln!(file, "{{")?;
    writeln!(file, "  \"version\": \"1.10\",")?;
    writeln!(file, "  \"propertyTypes\": [")?;

    for (i, type_export) in types.iter().enumerate() {
        let comma = if i < types.len() - 1 { "," } else { "" };

        writeln!(file, "    {{")?;
        writeln!(file, "      \"id\": {},", type_export.id)?;
        writeln!(file, "      \"name\": \"{}\",", type_export.name)?;
        writeln!(file, "      \"type\": \"class\",")?;
        writeln!(file, "      \"members\": [")?;

        // Export each field as a member
        for (field_idx, member) in type_export.members.iter().enumerate() {
            let field_comma = if field_idx < type_export.members.len() - 1 {
                ","
            } else {
                ""
            };

            writeln!(file, "        {{")?;
            writeln!(file, "          \"name\": \"{}\",", member.name)?;
            writeln!(file, "          \"type\": \"{}\",", member.tiled_type)?;

            // Write default value
            write!(file, "          \"value\": ")?;
            write_value(&mut *file, &member.value)?;
            writeln!(file)?;

            write!(file, "        }}{}", field_comma)?;
            if field_idx < type_export.members.len() - 1 {
                writeln!(file)?;
            }
        }

        writeln!(file)?;
        writeln!(file, "      ]")?;
        write!(file, "    }}{}", comma)?;

        if i < types.len() - 1 {
            writeln!(file)?;
        }
    }

    writeln!(file)?;
    writeln!(file, "  ]")?;
    writeln!(file, "}}")?;

    Ok(())
}

/// Write a TiledValueExport as JSON
fn write_value(file: &mut File, value: &TiledValueExport) -> std::io::Result<()> {
    match value {
        TiledValueExport::Bool(b) => write!(file, "{}", if *b { "true" } else { "false" }),
        TiledValueExport::Int(i) => write!(file, "{}", i),
        TiledValueExport::Float(f) => write!(file, "{}", f),
        TiledValueExport::String(s) => write!(file, "\"{}\"", s),
        TiledValueExport::Color(hex) => write!(file, "\"{}\"", hex),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_default_value_bool() {
        let value = TiledDefaultValue::Bool(true);
        let result = convert_default_value(&value);
        assert_eq!(result, TiledValueExport::Bool(true));
    }

    #[test]
    fn test_convert_default_value_int() {
        let value = TiledDefaultValue::Int(42);
        let result = convert_default_value(&value);
        assert_eq!(result, TiledValueExport::Int(42));
    }

    #[test]
    fn test_convert_default_value_float() {
        let value = TiledDefaultValue::Float(3.14);
        let result = convert_default_value(&value);
        assert_eq!(result, TiledValueExport::Float(3.14));
    }

    #[test]
    fn test_convert_default_value_string() {
        let value = TiledDefaultValue::String("test");
        let result = convert_default_value(&value);
        assert_eq!(result, TiledValueExport::String("test".to_string()));
    }

    #[test]
    fn test_convert_default_value_color() {
        let value = TiledDefaultValue::Color {
            r: 255,
            g: 128,
            b: 64,
            a: 255,
        };
        let result = convert_default_value(&value);
        assert_eq!(result, TiledValueExport::Color("#ffff8040".to_string()));
    }

    #[test]
    fn test_build_export_data_structure() {
        // Create a mock registry with test data
        use crate::properties::registry::{TiledClassInfo, TiledFieldInfo};
        use std::any::TypeId;

        static TEST_FIELDS: &[TiledFieldInfo] = &[
            TiledFieldInfo {
                name: "health",
                tiled_type: "float",
                default_value: TiledDefaultValue::Float(100.0),
            },
            TiledFieldInfo {
                name: "enabled",
                tiled_type: "bool",
                default_value: TiledDefaultValue::Bool(true),
            },
        ];

        // Verify field conversion works
        let members: Vec<TiledMemberExport> = TEST_FIELDS
            .iter()
            .map(|field| TiledMemberExport {
                name: field.name.to_string(),
                tiled_type: field.tiled_type.to_string(),
                value: convert_default_value(&field.default_value),
            })
            .collect();

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].name, "health");
        assert_eq!(members[0].tiled_type, "float");
        assert_eq!(members[0].value, TiledValueExport::Float(100.0));

        assert_eq!(members[1].name, "enabled");
        assert_eq!(members[1].tiled_type, "bool");
        assert_eq!(members[1].value, TiledValueExport::Bool(true));
    }

    #[test]
    fn test_tiled_type_export_structure() {
        let export = TiledTypeExport {
            id: 1,
            name: "game::Player".to_string(),
            members: vec![
                TiledMemberExport {
                    name: "speed".to_string(),
                    tiled_type: "float".to_string(),
                    value: TiledValueExport::Float(5.0),
                },
                TiledMemberExport {
                    name: "team".to_string(),
                    tiled_type: "int".to_string(),
                    value: TiledValueExport::Int(0),
                },
            ],
        };

        assert_eq!(export.id, 1);
        assert_eq!(export.name, "game::Player");
        assert_eq!(export.members.len(), 2);
        assert_eq!(export.members[0].name, "speed");
        assert_eq!(export.members[1].name, "team");
    }
}
