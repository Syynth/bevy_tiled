//! Export registered `TiledClass` types to JSON for Tiled editor integration.
//!
//! Generates a JSON file in the format expected by Tiled's custom property system.
//! See: <https://doc.mapeditor.org/en/stable/manual/custom-properties/>

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use bevy::app::App;
use bevy::prelude::*;
use bevy::reflect::{TypeInfo, TypeRegistry, TypeRegistration};

use super::registry::{TiledClassRegistry, TiledDefaultValue, TiledTypeKind};

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
    /// The base type: "bool", "int", "float", "string", "color", or "class"
    pub tiled_type: String,
    /// For class types, the full type path (e.g., "`glam::Vec2`", "`game::Door`")
    pub property_type: Option<String>,
    pub value: TiledValueExport,
}

/// Exportable default value (simplified for JSON serialization).
#[derive(Debug, Clone, PartialEq)]
pub enum TiledValueExport {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Color(String),   // Hex format: #AARRGGBB
    ClassDefault,    // Empty object {} for class types
}

/// Intermediate representation of a Tiled enum type for serialization.
#[derive(Debug, Clone, PartialEq)]
pub struct TiledEnumExport {
    pub id: usize,
    pub name: String,
    pub values: Vec<String>,  // Variant names
    pub values_as_flags: bool,  // Always false for now
}

/// Wrapper for either a class type or enum type export.
#[derive(Debug, Clone, PartialEq)]
pub enum TiledTypeOrEnumExport {
    Type(TiledTypeExport),
    Enum(TiledEnumExport),
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
                .map(|field| {
                    let (tiled_type, property_type, value) = match &field.tiled_type {
                        TiledTypeKind::Bool => ("bool".to_string(), None, convert_default_value(&field.default_value)),
                        TiledTypeKind::Int => ("int".to_string(), None, convert_default_value(&field.default_value)),
                        TiledTypeKind::Float => ("float".to_string(), None, convert_default_value(&field.default_value)),
                        TiledTypeKind::String => ("string".to_string(), None, convert_default_value(&field.default_value)),
                        TiledTypeKind::Color => ("color".to_string(), None, convert_default_value(&field.default_value)),
                        TiledTypeKind::Class { property_type } => {
                            // Check if this is actually an enum type
                            // Try exact match first, then fuzzy match by suffix
                            let is_enum = registry.get_enum(property_type).is_some()
                                || registry.enum_names().any(|name| {
                                    name.ends_with(&format!("::{}", property_type))
                                        || name == *property_type
                                });

                            if is_enum {
                                // It's an enum - export as string with propertyType
                                // Use the full name from the registry if available
                                let full_name = registry.enum_names()
                                    .find(|name| {
                                        name.ends_with(&format!("::{}", property_type))
                                            || *name == *property_type
                                    })
                                    .unwrap_or(property_type);
                                ("string".to_string(), Some(full_name.to_string()), TiledValueExport::String(String::new()))
                            } else {
                                // It's a class - use ClassDefault (empty object {})
                                ("class".to_string(), Some(property_type.to_string()), TiledValueExport::ClassDefault)
                            }
                        }
                        TiledTypeKind::Enum { property_type, .. } => {
                            // Enum fields are exported as string type with propertyType
                            ("string".to_string(), Some(property_type.to_string()), TiledValueExport::String(String::new()))
                        }
                    };

                    TiledMemberExport {
                        name: field.name.to_string(),
                        tiled_type,
                        property_type,
                        value,
                    }
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

/// Generate export data for all registered enum types.
///
/// This function converts enum registry entries into an intermediate representation
/// suitable for serialization.
///
/// # Arguments
///
/// * `registry` - The `TiledClassRegistry` containing all registered enums
///
/// # Returns
///
/// Vector of `TiledEnumExport` representing all registered enums
pub fn build_enum_export_data(registry: &TiledClassRegistry) -> Vec<TiledEnumExport> {
    registry
        .iter_enums()
        .enumerate()
        .map(|(i, enum_info)| TiledEnumExport {
            id: i + 1,
            name: enum_info.name.to_string(),
            values: enum_info.variants.iter().map(|s| s.to_string()).collect(),
            values_as_flags: false,
        })
        .collect()
}

/// Convert `TiledDefaultValue` to `TiledValueExport` for serialization.
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

            // Emit propertyType for class types
            if let Some(ref property_type) = member.property_type {
                writeln!(file, "          \"propertyType\": \"{}\",", property_type)?;
            }

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

/// Write a `TiledValueExport` as JSON
fn write_value(file: &mut File, value: &TiledValueExport) -> std::io::Result<()> {
    match value {
        TiledValueExport::Bool(b) => write!(file, "{}", if *b { "true" } else { "false" }),
        TiledValueExport::Int(i) => write!(file, "{}", i),
        TiledValueExport::Float(f) => write!(file, "{}", f),
        TiledValueExport::String(s) => write!(file, "\"{}\"", s),
        TiledValueExport::Color(hex) => write!(file, "\"{}\"", hex),
        TiledValueExport::ClassDefault => write!(file, "{{}}"),  // Empty object for class types
    }
}

/// Write mixed types and enums to file in Tiled's JSON format.
///
/// This function handles both class types and enum types in a single JSON output.
fn write_mixed_types_to_file(
    file: &mut File,
    items: &[TiledTypeOrEnumExport],
) -> std::io::Result<()> {
    writeln!(file, "{{")?;
    writeln!(file, "  \"version\": \"1.10\",")?;
    writeln!(file, "  \"propertyTypes\": [")?;

    for (i, item) in items.iter().enumerate() {
        let comma = if i < items.len() - 1 { "," } else { "" };

        match item {
            TiledTypeOrEnumExport::Type(type_export) => {
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

                    // Emit propertyType for class types
                    if let Some(ref property_type) = member.property_type {
                        writeln!(file, "          \"propertyType\": \"{}\",", property_type)?;
                    }

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
            }
            TiledTypeOrEnumExport::Enum(enum_export) => {
                writeln!(file, "    {{")?;
                writeln!(file, "      \"id\": {},", enum_export.id)?;
                writeln!(file, "      \"name\": \"{}\",", enum_export.name)?;
                writeln!(file, "      \"type\": \"enum\",")?;
                writeln!(file, "      \"values\": [")?;

                for (value_idx, variant) in enum_export.values.iter().enumerate() {
                    let value_comma = if value_idx < enum_export.values.len() - 1 {
                        ","
                    } else {
                        ""
                    };
                    writeln!(file, "        \"{}\"{}",  variant, value_comma)?;
                }

                writeln!(file, "      ],")?;
                writeln!(
                    file,
                    "      \"valuesAsFlags\": {}",
                    if enum_export.values_as_flags {
                        "true"
                    } else {
                        "false"
                    }
                )?;
                write!(file, "    }}{}", comma)?;
            }
        }

        if i < items.len() - 1 {
            writeln!(file)?;
        }
    }

    writeln!(file)?;
    writeln!(file, "  ]")?;
    writeln!(file, "}}")?;

    Ok(())
}

// ============================================================================
// Reflection-based Export (Hybrid Approach)
// ============================================================================

/// Export all types using hybrid approach: `TiledClass` registry + Bevy reflection.
///
/// This function discovers types transitively:
/// 1. Starts with all TiledClass-registered types
/// 2. For each type, recursively discovers referenced types
/// 3. Uses `TiledClassRegistry` for manually registered types (primary)
/// 4. Falls back to Bevy's `AppTypeRegistry` for other types (secondary)
///
/// # Arguments
///
/// * `app` - The Bevy App instance (for accessing `AppTypeRegistry`)
/// * `output_path` - Path where the JSON file should be written
///
/// # Example
///
/// ```ignore
/// export_all_types_with_reflection(&app, "assets/custom-types.json")?;
/// ```
pub fn export_all_types_with_reflection(
    app: &App,
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let mut discovered_types = HashSet::new();
    let mut all_exports = Vec::new();

    // Start with all TiledClass types
    let tiled_registry = app.world().resource::<TiledClassRegistry>();

    // Export class types
    let type_names: Vec<String> = tiled_registry.type_names().map(ToString::to_string).collect();
    for type_name in type_names {
        discover_type_recursive(
            &type_name,
            app,
            &mut discovered_types,
            &mut all_exports,
        );
    }

    // Export enum types
    let enum_exports = build_enum_export_data(tiled_registry);
    all_exports.extend(enum_exports.into_iter().map(TiledTypeOrEnumExport::Enum));

    // Renumber IDs sequentially
    for (i, item) in all_exports.iter_mut().enumerate() {
        match item {
            TiledTypeOrEnumExport::Type(type_export) => type_export.id = i + 1,
            TiledTypeOrEnumExport::Enum(enum_export) => enum_export.id = i + 1,
        }
    }

    // Write to file
    let path = output_path.as_ref();
    let mut file = File::create(path)?;
    write_mixed_types_to_file(&mut file, &all_exports)?;

    Ok(())
}

/// Recursively discover a type and all its referenced types.
///
/// Uses hybrid lookup: `TiledClass` registry first, then Bevy reflection.
fn discover_type_recursive(
    type_path: &str,
    app: &App,
    discovered: &mut HashSet<String>,
    output: &mut Vec<TiledTypeOrEnumExport>,
) {
    // Skip if already processed
    if discovered.contains(type_path) {
        return;
    }
    discovered.insert(type_path.to_string());

    // Try TiledClass registry first
    let tiled_registry = app.world().resource::<TiledClassRegistry>();
    if let Some(tiled_class) = tiled_registry.get(type_path) {
        // Build export from TiledClass
        let members: Vec<TiledMemberExport> = tiled_class
            .fields
            .iter()
            .map(|field| {
                let (tiled_type, property_type, value) = match &field.tiled_type {
                    TiledTypeKind::Bool => ("bool".to_string(), None, convert_default_value(&field.default_value)),
                    TiledTypeKind::Int => ("int".to_string(), None, convert_default_value(&field.default_value)),
                    TiledTypeKind::Float => ("float".to_string(), None, convert_default_value(&field.default_value)),
                    TiledTypeKind::String => ("string".to_string(), None, convert_default_value(&field.default_value)),
                    TiledTypeKind::Color => ("color".to_string(), None, convert_default_value(&field.default_value)),
                    TiledTypeKind::Class { property_type } => {
                        // Check if this is actually an enum type
                        // Try exact match first, then fuzzy match by suffix
                        let is_enum = tiled_registry.get_enum(property_type).is_some()
                            || tiled_registry.enum_names().any(|name| {
                                name.ends_with(&format!("::{}", property_type))
                                    || name == *property_type
                            });

                        if is_enum {
                            // It's an enum - export as string with propertyType
                            // Use the full name from the registry if available
                            let full_name = tiled_registry.enum_names()
                                .find(|name| {
                                    name.ends_with(&format!("::{}", property_type))
                                        || *name == *property_type
                                })
                                .unwrap_or(property_type);
                            ("string".to_string(), Some(full_name.to_string()), TiledValueExport::String(String::new()))
                        } else {
                            // It's a class - use ClassDefault (empty object {})
                            ("class".to_string(), Some(property_type.to_string()), TiledValueExport::ClassDefault)
                        }
                    }
                    TiledTypeKind::Enum { property_type, .. } => {
                        // Enum types are exported as string with propertyType reference
                        ("string".to_string(), Some(property_type.to_string()), TiledValueExport::String(String::new()))
                    }
                };

                TiledMemberExport {
                    name: field.name.to_string(),
                    tiled_type,
                    property_type,
                    value,
                }
            })
            .collect();

        output.push(TiledTypeOrEnumExport::Type(TiledTypeExport {
            id: 0, // Will be renumbered later
            name: tiled_class.name.to_string(),
            members,
        }));

        // Recursively discover referenced types
        for field in tiled_class.fields {
            match &field.tiled_type {
                TiledTypeKind::Class { property_type } | TiledTypeKind::Enum { property_type, .. } => {
                    discover_type_recursive(property_type, app, discovered, output);
                }
                _ => {}
            }
        }
        return;
    }

    // Fall back to Bevy reflection
    let app_type_registry = app.world().resource::<AppTypeRegistry>();
    let registry = app_type_registry.read();

    if let Some(reflect_type) = registry.get_with_type_path(type_path) {
        if let Some(export) = build_reflected_export(reflect_type, &registry) {
            output.push(TiledTypeOrEnumExport::Type(export));

            // Recursively discover reflected field types
            if let TypeInfo::Struct(struct_info) = reflect_type.type_info() {
                for field in struct_info.iter() {
                    let field_type_path = field.type_path();
                    if !is_primitive_type(field_type_path) {
                        discover_type_recursive(field_type_path, app, discovered, output);
                    }
                }
            }
        }
        return;
    }

    // Type not found - warn but don't fail
    warn!(
        "Referenced type '{}' not found in TiledClassRegistry or AppTypeRegistry",
        type_path
    );
}

/// Build a `TiledTypeExport` from a reflected type.
///
/// Returns None if the type is not a struct or doesn't have fields.
fn build_reflected_export(
    reflect_type: &TypeRegistration,
    _registry: &TypeRegistry,
) -> Option<TiledTypeExport> {
    let type_info = reflect_type.type_info();

    let TypeInfo::Struct(struct_info) = type_info else {
        return None;
    };

    let members: Vec<TiledMemberExport> = struct_info
        .iter()
        .map(|field| {
            let field_type_path = field.type_path();
            let (tiled_type, property_type, value) = if is_primitive_type(field_type_path) {
                let tiled_type = map_primitive_to_tiled(field_type_path);
                // Generate appropriate default value for primitives
                let default_value = match tiled_type.as_str() {
                    "bool" => TiledValueExport::Bool(false),
                    "int" => TiledValueExport::Int(0),
                    "float" => TiledValueExport::Float(0.0),
                    "string" | _ => TiledValueExport::String(String::new()),
                };
                (tiled_type, None, default_value)
            } else {
                ("class".to_string(), Some(field_type_path.to_string()), TiledValueExport::ClassDefault)
            };

            TiledMemberExport {
                name: field.name().to_string(),
                tiled_type,
                property_type,
                value,
            }
        })
        .collect();

    Some(TiledTypeExport {
        id: 0, // Will be renumbered later
        name: type_info.type_path().to_string(),
        members,
    })
}

/// Check if a type path represents a primitive Tiled type.
fn is_primitive_type(type_path: &str) -> bool {
    matches!(
        type_path,
        "bool"
            | "i32"
            | "i64"
            | "u32"
            | "u64"
            | "f32"
            | "f64"
            | "alloc::string::String"
            | "&str"
    )
}

/// Map a primitive Rust type path to a Tiled type string.
fn map_primitive_to_tiled(type_path: &str) -> String {
    match type_path {
        "bool" => "bool",
        "i32" | "i64" | "u32" | "u64" => "int",
        "f32" | "f64" => "float",
        "alloc::string::String" | "&str" | _ => "string",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

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
        let value = TiledDefaultValue::Float(PI);
        let result = convert_default_value(&value);
        assert_eq!(result, TiledValueExport::Float(PI));
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
        use crate::properties::registry::TiledFieldInfo;

        static TEST_FIELDS: &[TiledFieldInfo] = &[
            TiledFieldInfo {
                name: "health",
                tiled_type: TiledTypeKind::Float,
                default_value: TiledDefaultValue::Float(100.0),
            },
            TiledFieldInfo {
                name: "enabled",
                tiled_type: TiledTypeKind::Bool,
                default_value: TiledDefaultValue::Bool(true),
            },
        ];

        // Verify field conversion works
        let members: Vec<TiledMemberExport> = TEST_FIELDS
            .iter()
            .map(|field| {
                let (tiled_type, property_type) = match &field.tiled_type {
                    TiledTypeKind::Bool => ("bool".to_string(), None),
                    TiledTypeKind::Float => ("float".to_string(), None),
                    _ => ("string".to_string(), None),
                };
                TiledMemberExport {
                    name: field.name.to_string(),
                    tiled_type,
                    property_type,
                    value: convert_default_value(&field.default_value),
                }
            })
            .collect();

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].name, "health");
        assert_eq!(members[0].tiled_type, "float");
        assert_eq!(members[0].property_type, None);
        assert_eq!(members[0].value, TiledValueExport::Float(100.0));

        assert_eq!(members[1].name, "enabled");
        assert_eq!(members[1].tiled_type, "bool");
        assert_eq!(members[1].property_type, None);
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
                    property_type: None,
                    value: TiledValueExport::Float(5.0),
                },
                TiledMemberExport {
                    name: "team".to_string(),
                    tiled_type: "int".to_string(),
                    property_type: None,
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
