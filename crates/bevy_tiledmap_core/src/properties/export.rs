//! Export registered `TiledClass` types to JSON for Tiled editor integration.
//!
//! Generates a JSON file in the format expected by Tiled's custom property system.
//! See: <https://doc.mapeditor.org/en/stable/manual/custom-properties/>

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use bevy::prelude::*;
use bevy::reflect::{TypeInfo, TypeRegistration, TypeRegistry};

use super::registry::{
    TiledClassRegistry, TiledDefaultValue, TiledEnumInfo, TiledEnumKind, TiledTypeKind,
    TiledVariantKind,
};

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
    Color(String), // Hex format: #AARRGGBB
    ClassDefault,  // Empty object {} for class types
}

/// Intermediate representation of a Tiled enum type for serialization.
#[derive(Debug, Clone, PartialEq)]
pub struct TiledEnumExport {
    pub id: usize,
    pub name: String,
    pub values: Vec<String>,   // Variant names
    pub values_as_flags: bool, // Always false for now
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
                        TiledTypeKind::Bool => (
                            "bool".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::Int => (
                            "int".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::Float => (
                            "float".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::String => (
                            "string".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::Color => (
                            "color".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::File => (
                            "file".to_string(),
                            None,
                            TiledValueExport::String(String::new()),
                        ),
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
                                let full_name = registry
                                    .enum_names()
                                    .find(|name| {
                                        name.ends_with(&format!("::{}", property_type))
                                            || *name == *property_type
                                    })
                                    .unwrap_or(property_type);
                                (
                                    "string".to_string(),
                                    Some(full_name.to_string()),
                                    TiledValueExport::String(String::new()),
                                )
                            } else {
                                // It's a class - use ClassDefault (empty object {})
                                (
                                    "class".to_string(),
                                    Some(property_type.to_string()),
                                    TiledValueExport::ClassDefault,
                                )
                            }
                        }
                        TiledTypeKind::Enum { property_type, .. } => {
                            // Enum fields are exported as string type with propertyType
                            (
                                "string".to_string(),
                                Some(property_type.to_string()),
                                TiledValueExport::String(String::new()),
                            )
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
/// suitable for serialization. Enums are sorted by name for deterministic ID assignment.
///
/// # Arguments
///
/// * `registry` - The `TiledClassRegistry` containing all registered enums
///
/// # Returns
///
/// Vector of `TiledEnumExport` representing all registered enums
pub fn build_enum_export_data(registry: &TiledClassRegistry) -> Vec<TiledEnumExport> {
    // Collect and sort enums by name for deterministic ordering
    let mut enums: Vec<_> = registry.iter_enums().collect();
    enums.sort_by_key(|e| e.name);

    enums
        .iter()
        .enumerate()
        .filter_map(|(i, enum_info)| {
            // Only export simple enums here
            // Complex enums are exported as class types in build_export_data
            if enum_info.is_simple() {
                Some(TiledEnumExport {
                    id: i + 1,
                    name: enum_info.name.to_string(),
                    values: enum_info
                        .variant_names()
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    values_as_flags: false,
                })
            } else {
                None
            }
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

/// Export a complex enum as a Tiled class type with `:variant` discriminant field.
///
/// Complex enums (with struct/tuple variants) are exported as class types rather than
/// simple enums. The class includes a `:variant` field (string type) that acts as the
/// discriminant, plus the union of all fields from all variants.
///
/// # Arguments
///
/// * `enum_info` - The enum information from the registry
/// * `id` - The ID to assign to this exported type
/// * `registry` - The registry (for checking if referenced types are enums)
///
/// # Returns
///
/// A `TiledTypeExport` representing the complex enum as a class type
fn export_complex_enum(
    enum_info: &TiledEnumInfo,
    id: usize,
    registry: &TiledClassRegistry,
) -> TiledTypeExport {
    let variant_info = match &enum_info.kind {
        TiledEnumKind::Complex { variant_info } => variant_info,
        TiledEnumKind::Simple { .. } => {
            panic!("export_complex_enum called on simple enum");
        }
    };

    let mut members = Vec::new();
    let mut field_types = HashMap::new();

    // Add :variant discriminant field first
    let default_variant = enum_info.default_variant_name().unwrap_or("");
    members.push(TiledMemberExport {
        name: ":variant".to_string(),
        tiled_type: "string".to_string(),
        property_type: Some(format!("{}:::variant", enum_info.name)),
        value: TiledValueExport::String(default_variant.to_string()),
    });

    // Collect union of all variant fields
    for variant in variant_info.iter() {
        if let Some(fields_info) = match &variant.kind {
            TiledVariantKind::Unit => None,
            TiledVariantKind::Struct { fields } | TiledVariantKind::Tuple { fields } => {
                Some(*fields)
            }
        } {
            for field in fields_info {
                // Check for type conflicts
                if let Some(existing_type) = field_types.get(field.name) {
                    if !types_match(existing_type, &field.tiled_type) {
                        warn!(
                            "Field '{}' has conflicting types in enum '{}': {:?} vs {:?}. Using first type.",
                            field.name, enum_info.name, existing_type, field.tiled_type
                        );
                        continue;
                    }
                } else {
                    field_types.insert(field.name.to_string(), field.tiled_type.clone());

                    // Export the field
                    let (tiled_type, property_type, value) = match &field.tiled_type {
                        TiledTypeKind::Bool => (
                            "bool".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::Int => (
                            "int".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::Float => (
                            "float".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::String => (
                            "string".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::Color => (
                            "color".to_string(),
                            None,
                            convert_default_value(&field.default_value),
                        ),
                        TiledTypeKind::File => (
                            "file".to_string(),
                            None,
                            TiledValueExport::String(String::new()),
                        ),
                        TiledTypeKind::Class { property_type } => {
                            // Check if this is an enum
                            let is_enum = registry.get_enum(property_type).is_some();

                            if is_enum {
                                (
                                    "string".to_string(),
                                    Some(property_type.to_string()),
                                    TiledValueExport::String(String::new()),
                                )
                            } else {
                                (
                                    "class".to_string(),
                                    Some(property_type.to_string()),
                                    TiledValueExport::ClassDefault,
                                )
                            }
                        }
                        TiledTypeKind::Enum { property_type, .. } => (
                            "string".to_string(),
                            Some(property_type.to_string()),
                            TiledValueExport::String(String::new()),
                        ),
                    };

                    members.push(TiledMemberExport {
                        name: field.name.to_string(),
                        tiled_type,
                        property_type,
                        value,
                    });
                }
            }
        }
    }

    TiledTypeExport {
        id,
        name: enum_info.name.to_string(),
        members,
    }
}

/// Check if two `TiledTypeKind` values represent the same type.
fn types_match(a: &TiledTypeKind, b: &TiledTypeKind) -> bool {
    match (a, b) {
        (TiledTypeKind::Bool, TiledTypeKind::Bool)
        | (TiledTypeKind::Int, TiledTypeKind::Int)
        | (TiledTypeKind::Float, TiledTypeKind::Float)
        | (TiledTypeKind::String, TiledTypeKind::String)
        | (TiledTypeKind::Color, TiledTypeKind::Color) => true,
        (TiledTypeKind::Class { property_type: a }, TiledTypeKind::Class { property_type: b }) => {
            a == b
        }
        (
            TiledTypeKind::Enum {
                property_type: a, ..
            },
            TiledTypeKind::Enum {
                property_type: b, ..
            },
        ) => a == b,
        _ => false,
    }
}

/// Generate a synthetic enum for the `:variant` discriminant field.
///
/// This creates an enum named `EnumName:::variant` with all the variant names
/// as values. This allows Tiled to show a dropdown for selecting variants.
///
/// # Arguments
///
/// * `enum_info` - The enum information from the registry
/// * `id` - The ID to assign to this exported enum
///
/// # Returns
///
/// A `TiledEnumExport` for the synthetic variant selector enum
fn generate_variant_names_enum(enum_info: &TiledEnumInfo, id: usize) -> TiledEnumExport {
    let variant_names = enum_info.variant_names();

    TiledEnumExport {
        id,
        name: format!("{}:::variant", enum_info.name),
        values: variant_names.iter().map(ToString::to_string).collect(),
        values_as_flags: false,
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

    // Build exports for both classes and enums
    let mut all_exports = Vec::new();

    // Export class types
    let class_exports = build_export_data(registry);
    all_exports.extend(class_exports.into_iter().map(TiledTypeOrEnumExport::Type));

    // Export enum types
    let enum_exports = build_enum_export_data(registry);
    all_exports.extend(enum_exports.into_iter().map(TiledTypeOrEnumExport::Enum));

    // Renumber IDs sequentially
    for (i, item) in all_exports.iter_mut().enumerate() {
        match item {
            TiledTypeOrEnumExport::Type(type_export) => type_export.id = i + 1,
            TiledTypeOrEnumExport::Enum(enum_export) => enum_export.id = i + 1,
        }
    }

    write_mixed_types_to_file(&mut file, &all_exports)?;

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
        TiledValueExport::ClassDefault => write!(file, "null"), // null for class types
    }
}

/// Write mixed types and enums to file in Tiled's JSON format.
///
/// This function handles both class types and enum types in a single JSON output.
fn write_mixed_types_to_file(
    file: &mut File,
    items: &[TiledTypeOrEnumExport],
) -> std::io::Result<()> {
    writeln!(file, "[")?;

    for (i, item) in items.iter().enumerate() {
        let comma = if i < items.len() - 1 { "," } else { "" };

        match item {
            TiledTypeOrEnumExport::Type(type_export) => {
                writeln!(file, "  {{")?;
                writeln!(file, "    \"id\": {},", type_export.id)?;
                writeln!(file, "    \"name\": \"{}\",", type_export.name)?;
                writeln!(file, "    \"type\": \"class\",")?;
                writeln!(file, "    \"useAs\": [")?;
                writeln!(file, "      \"property\"")?;
                writeln!(file, "    ],")?;
                writeln!(file, "    \"color\": \"#000000\",")?;
                writeln!(file, "    \"drawFill\": true,")?;
                writeln!(file, "    \"members\": [")?;

                // Export each field as a member
                for (field_idx, member) in type_export.members.iter().enumerate() {
                    let field_comma = if field_idx < type_export.members.len() - 1 {
                        ","
                    } else {
                        ""
                    };

                    writeln!(file, "      {{")?;
                    writeln!(file, "        \"name\": \"{}\",", member.name)?;

                    // Emit propertyType for class types (before type)
                    if let Some(ref property_type) = member.property_type {
                        writeln!(file, "        \"propertyType\": \"{}\",", property_type)?;
                    }

                    writeln!(file, "        \"type\": \"{}\",", member.tiled_type)?;

                    // Write default value
                    write!(file, "        \"value\": ")?;
                    write_value(&mut *file, &member.value)?;
                    writeln!(file)?;

                    write!(file, "      }}{}", field_comma)?;
                    if field_idx < type_export.members.len() - 1 {
                        writeln!(file)?;
                    }
                }

                writeln!(file)?;
                writeln!(file, "    ]")?;
                write!(file, "  }}{}", comma)?;
            }
            TiledTypeOrEnumExport::Enum(enum_export) => {
                writeln!(file, "  {{")?;
                writeln!(file, "    \"id\": {},", enum_export.id)?;
                writeln!(file, "    \"name\": \"{}\",", enum_export.name)?;
                writeln!(file, "    \"type\": \"enum\",")?;
                writeln!(file, "    \"storageType\": \"string\",")?;
                writeln!(file, "    \"values\": [")?;

                for (value_idx, variant) in enum_export.values.iter().enumerate() {
                    let value_comma = if value_idx < enum_export.values.len() - 1 {
                        ","
                    } else {
                        ""
                    };
                    writeln!(file, "      \"{}\"{}", variant, value_comma)?;
                }

                writeln!(file, "    ],")?;
                writeln!(
                    file,
                    "    \"valuesAsFlags\": {}",
                    if enum_export.values_as_flags {
                        "true"
                    } else {
                        "false"
                    }
                )?;
                write!(file, "  }}{}", comma)?;
            }
        }

        if i < items.len() - 1 {
            writeln!(file)?;
        }
    }

    writeln!(file)?;
    writeln!(file, "]")?;

    Ok(())
}

// ============================================================================
// ID Preservation for Stable Exports
// ============================================================================

/// Read existing type/enum IDs from a JSON file.
///
/// Returns a mapping of type name to ID. If the file doesn't exist or is invalid,
/// returns an empty map.
fn read_existing_ids(path: &Path) -> HashMap<String, usize> {
    let Ok(content) = fs::read_to_string(path) else {
        return HashMap::new();
    };

    let Ok(json): Result<serde_json::Value, _> = serde_json::from_str(&content) else {
        return HashMap::new();
    };

    let mut ids = HashMap::new();

    // Handle both standalone array format and .tiled-project format
    let property_types = if let Some(arr) = json.as_array() {
        // Standalone JSON array format
        arr.clone()
    } else if let Some(obj) = json.as_object() {
        // .tiled-project format with propertyTypes field
        obj.get("propertyTypes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    } else {
        return HashMap::new();
    };

    for item in property_types {
        if let (Some(name), Some(id)) = (
            item.get("name").and_then(serde_json::Value::as_str),
            item.get("id").and_then(serde_json::Value::as_u64),
        ) {
            ids.insert(name.to_string(), id as usize);
        }
    }

    ids
}

/// Assign IDs to exports, preserving existing IDs and filling gaps for new types.
///
/// This function:
/// 1. Assigns existing IDs to types that already have them
/// 2. Collects gaps in the ID sequence (unused IDs between 1 and max)
/// 3. Assigns new types to gap IDs first
/// 4. Continues counting from `max_id` + 1 for any remaining new types
fn assign_ids_with_preservation(
    exports: &mut [TiledTypeOrEnumExport],
    existing_ids: &HashMap<String, usize>,
) {
    // Track which IDs are used
    let mut used_ids = BTreeSet::new();
    let mut max_id = 0usize;

    // First pass: assign existing IDs to matching types
    for export in exports.iter_mut() {
        let name = match export {
            TiledTypeOrEnumExport::Type(t) => &t.name,
            TiledTypeOrEnumExport::Enum(e) => &e.name,
        };

        if let Some(&existing_id) = existing_ids.get(name) {
            match export {
                TiledTypeOrEnumExport::Type(t) => t.id = existing_id,
                TiledTypeOrEnumExport::Enum(e) => e.id = existing_id,
            }
            used_ids.insert(existing_id);
            max_id = max_id.max(existing_id);
        }
    }

    // Find gaps in the sequence (IDs between 1 and max that are unused)
    let mut gaps: Vec<usize> = (1..=max_id).filter(|id| !used_ids.contains(id)).collect();
    gaps.reverse(); // So we can pop from the end

    // Second pass: assign IDs to new types using gaps first, then incrementing
    let mut next_id = max_id + 1;
    for export in exports.iter_mut() {
        let current_id = match export {
            TiledTypeOrEnumExport::Type(t) => t.id,
            TiledTypeOrEnumExport::Enum(e) => e.id,
        };

        // If ID is still 0, it's a new type that needs an ID
        if current_id == 0 {
            let new_id = gaps.pop().unwrap_or_else(|| {
                let id = next_id;
                next_id += 1;
                id
            });

            match export {
                TiledTypeOrEnumExport::Type(t) => t.id = new_id,
                TiledTypeOrEnumExport::Enum(e) => e.id = new_id,
            }
        }
    }
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
/// * `world` - The Bevy World (for accessing `AppTypeRegistry`)
/// * `output_path` - Path where the JSON file should be written
///
/// # Example
///
/// ```ignore
/// export_all_types_with_reflection(world, "assets/custom-types.json")?;
/// ```
pub fn export_all_types_with_reflection(
    world: &World,
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let mut discovered_types = HashSet::new();
    let mut all_exports = Vec::new();

    // Start with all TiledClass types
    let tiled_registry = world.resource::<TiledClassRegistry>();

    // Export class types (sorted for deterministic ID assignment)
    let mut type_names: Vec<String> = tiled_registry
        .type_names()
        .map(ToString::to_string)
        .collect();
    type_names.sort();
    for type_name in type_names {
        discover_type_recursive(&type_name, world, &mut discovered_types, &mut all_exports);
    }

    // Export enum types
    let simple_enum_exports = build_enum_export_data(tiled_registry);
    all_exports.extend(
        simple_enum_exports
            .into_iter()
            .map(TiledTypeOrEnumExport::Enum),
    );

    // Export complex enums (as class types) and their synthetic variant enums
    // Sort by name for deterministic ordering
    let mut complex_enums: Vec<_> = tiled_registry
        .iter_enums()
        .filter(|e| e.is_complex())
        .collect();
    complex_enums.sort_by_key(|e| e.name);

    let mut next_id = all_exports.len() + 1;
    for enum_info in complex_enums {
        // Export the complex enum as a class type
        let complex_export = export_complex_enum(enum_info, next_id, tiled_registry);
        all_exports.push(TiledTypeOrEnumExport::Type(complex_export));
        next_id += 1;

        // Export the synthetic :::variant enum
        let variant_enum = generate_variant_names_enum(enum_info, next_id);
        all_exports.push(TiledTypeOrEnumExport::Enum(variant_enum));
        next_id += 1;

        // Recursively discover referenced types in variant fields
        if let Some(variant_info_slice) = enum_info.variant_info() {
            for variant in variant_info_slice {
                if let Some(fields) = match &variant.kind {
                    TiledVariantKind::Unit => None,
                    TiledVariantKind::Struct { fields } | TiledVariantKind::Tuple { fields } => {
                        Some(*fields)
                    }
                } {
                    for field in fields {
                        if let TiledTypeKind::Class { property_type } = &field.tiled_type {
                            discover_type_recursive(
                                property_type,
                                world,
                                &mut discovered_types,
                                &mut all_exports,
                            );
                        }
                    }
                }
            }
        }
    }

    // Assign IDs, preserving existing ones from file if it exists
    let path = output_path.as_ref();
    let existing_ids = read_existing_ids(path);
    assign_ids_with_preservation(&mut all_exports, &existing_ids);

    // Write to file
    let mut file = File::create(path)?;
    write_mixed_types_to_file(&mut file, &all_exports)?;

    Ok(())
}

/// Export types directly to a `.tiled-project` file.
///
/// This function updates the `propertyTypes` array in the project file while
/// preserving all other fields (folders, commands, compatibilityVersion, etc.).
///
/// Merge behavior:
/// - Existing types that match Rust types (by name) are updated with new definitions
/// - Manually-added types that don't match any Rust type are preserved
/// - New Rust types are added with IDs that fill gaps or increment from max
///
/// # Arguments
///
/// * `world` - The Bevy World (for accessing registries)
/// * `project_path` - Path to the `.tiled-project` file
///
/// # Example
///
/// ```ignore
/// export_to_tiled_project(world, "assets/my.tiled-project")?;
/// ```
pub fn export_to_tiled_project(
    world: &World,
    project_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let path = project_path.as_ref();

    // Read existing project file or create default structure
    let mut project_json: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
    } else {
        // Create a minimal default project structure
        serde_json::json!({
            "automappingRulesFile": "",
            "commands": [],
            "compatibilityVersion": 1100,
            "extensionsPath": "extensions",
            "folders": ["."],
            "properties": [],
            "propertyTypes": []
        })
    };

    // Build all exports using the same logic as export_all_types_with_reflection
    let mut all_exports = build_all_exports(world);

    // Read existing IDs from the project file
    let existing_ids = read_existing_ids(path);

    // Also track which names exist in the project but not in our exports
    // (manually-added types that should be preserved)
    let exported_names: HashSet<String> = all_exports
        .iter()
        .map(|e| match e {
            TiledTypeOrEnumExport::Type(t) => t.name.clone(),
            TiledTypeOrEnumExport::Enum(e) => e.name.clone(),
        })
        .collect();

    // Get manually-added types from existing project
    let manual_types: Vec<serde_json::Value> = project_json
        .get("propertyTypes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|item| {
                    item.get("name")
                        .and_then(|n| n.as_str())
                        .map(|name| !exported_names.contains(name))
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    // Add manual type IDs to existing_ids so they're considered when assigning new IDs
    let mut all_existing_ids = existing_ids;
    for item in &manual_types {
        if let (Some(name), Some(id)) = (
            item.get("name").and_then(serde_json::Value::as_str),
            item.get("id").and_then(serde_json::Value::as_u64),
        ) {
            all_existing_ids.insert(name.to_string(), id as usize);
        }
    }

    // Assign IDs with preservation
    assign_ids_with_preservation(&mut all_exports, &all_existing_ids);

    // Convert exports to JSON values
    let mut property_types: Vec<serde_json::Value> = Vec::new();

    for export in &all_exports {
        let json_value = match export {
            TiledTypeOrEnumExport::Type(t) => export_type_to_json(t),
            TiledTypeOrEnumExport::Enum(e) => export_enum_to_json(e),
        };
        property_types.push(json_value);
    }

    // Add back manually-added types
    property_types.extend(manual_types);

    // Sort by ID for clean output
    property_types.sort_by_key(|v| v.get("id").and_then(serde_json::Value::as_u64).unwrap_or(0));

    // Update the propertyTypes array in the project
    project_json["propertyTypes"] = serde_json::Value::Array(property_types);

    // Write back with pretty formatting
    let output = serde_json::to_string_pretty(&project_json)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    fs::write(path, output)?;

    Ok(())
}

/// Build all exports without writing to a file.
///
/// This is a helper that extracts the export-building logic for reuse.
fn build_all_exports(world: &World) -> Vec<TiledTypeOrEnumExport> {
    let mut discovered_types = HashSet::new();
    let mut all_exports = Vec::new();

    let tiled_registry = world.resource::<TiledClassRegistry>();

    // Export class types (sorted for deterministic ordering)
    let mut type_names: Vec<String> = tiled_registry
        .type_names()
        .map(ToString::to_string)
        .collect();
    type_names.sort();
    for type_name in type_names {
        discover_type_recursive(&type_name, world, &mut discovered_types, &mut all_exports);
    }

    // Export simple enum types (sorted)
    let simple_enum_exports = build_enum_export_data(tiled_registry);
    all_exports.extend(
        simple_enum_exports
            .into_iter()
            .map(TiledTypeOrEnumExport::Enum),
    );

    // Export complex enums (sorted)
    let mut complex_enums: Vec<_> = tiled_registry
        .iter_enums()
        .filter(|e| e.is_complex())
        .collect();
    complex_enums.sort_by_key(|e| e.name);

    let mut next_id = all_exports.len() + 1;
    for enum_info in complex_enums {
        let complex_export = export_complex_enum(enum_info, next_id, tiled_registry);
        all_exports.push(TiledTypeOrEnumExport::Type(complex_export));
        next_id += 1;

        let variant_enum = generate_variant_names_enum(enum_info, next_id);
        all_exports.push(TiledTypeOrEnumExport::Enum(variant_enum));
        next_id += 1;

        if let Some(variant_info_slice) = enum_info.variant_info() {
            for variant in variant_info_slice {
                if let Some(fields) = match &variant.kind {
                    TiledVariantKind::Unit => None,
                    TiledVariantKind::Struct { fields } | TiledVariantKind::Tuple { fields } => {
                        Some(*fields)
                    }
                } {
                    for field in fields {
                        if let TiledTypeKind::Class { property_type } = &field.tiled_type {
                            discover_type_recursive(
                                property_type,
                                world,
                                &mut discovered_types,
                                &mut all_exports,
                            );
                        }
                    }
                }
            }
        }
    }

    all_exports
}

/// Convert a [`TiledTypeExport`] to a [`serde_json::Value`] for the project file.
fn export_type_to_json(t: &TiledTypeExport) -> serde_json::Value {
    let members: Vec<serde_json::Value> = t
        .members
        .iter()
        .map(|m| {
            let mut member = serde_json::json!({
                "name": m.name,
                "type": m.tiled_type,
                "value": value_to_json(&m.value)
            });
            if let Some(ref pt) = m.property_type {
                member["propertyType"] = serde_json::Value::String(pt.clone());
            }
            member
        })
        .collect();

    serde_json::json!({
        "id": t.id,
        "name": t.name,
        "type": "class",
        "color": "#ff000000",
        "drawFill": true,
        "members": members,
        "useAs": ["property"]
    })
}

/// Convert a [`TiledEnumExport`] to a [`serde_json::Value`] for the project file.
fn export_enum_to_json(e: &TiledEnumExport) -> serde_json::Value {
    serde_json::json!({
        "id": e.id,
        "name": e.name,
        "type": "enum",
        "storageType": "string",
        "values": e.values,
        "valuesAsFlags": e.values_as_flags
    })
}

/// Convert a [`TiledValueExport`] to a [`serde_json::Value`].
fn value_to_json(value: &TiledValueExport) -> serde_json::Value {
    match value {
        TiledValueExport::Bool(b) => serde_json::Value::Bool(*b),
        TiledValueExport::Int(i) => serde_json::json!(*i),
        TiledValueExport::Float(f) => serde_json::json!(*f),
        TiledValueExport::String(s) => serde_json::Value::String(s.clone()),
        TiledValueExport::Color(hex) => serde_json::Value::String(hex.clone()),
        TiledValueExport::ClassDefault => serde_json::Value::Null,
    }
}

/// Recursively discover a type and all its referenced types.
///
/// Uses hybrid lookup: `TiledClass` registry first, then Bevy reflection.
fn discover_type_recursive(
    type_path: &str,
    world: &World,
    discovered: &mut HashSet<String>,
    output: &mut Vec<TiledTypeOrEnumExport>,
) {
    // Skip if already processed
    if discovered.contains(type_path) {
        return;
    }
    discovered.insert(type_path.to_string());

    // Try TiledClass registry first
    let tiled_registry = world.resource::<TiledClassRegistry>();
    if let Some(tiled_class) = tiled_registry.get(type_path) {
        // Build export from TiledClass
        let members: Vec<TiledMemberExport> = tiled_class
            .fields
            .iter()
            .map(|field| {
                let (tiled_type, property_type, value) = match &field.tiled_type {
                    TiledTypeKind::Bool => (
                        "bool".to_string(),
                        None,
                        convert_default_value(&field.default_value),
                    ),
                    TiledTypeKind::Int => (
                        "int".to_string(),
                        None,
                        convert_default_value(&field.default_value),
                    ),
                    TiledTypeKind::Float => (
                        "float".to_string(),
                        None,
                        convert_default_value(&field.default_value),
                    ),
                    TiledTypeKind::String => (
                        "string".to_string(),
                        None,
                        convert_default_value(&field.default_value),
                    ),
                    TiledTypeKind::Color => (
                        "color".to_string(),
                        None,
                        convert_default_value(&field.default_value),
                    ),
                    TiledTypeKind::File => (
                        "file".to_string(),
                        None,
                        TiledValueExport::String(String::new()),
                    ),
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
                            let full_name = tiled_registry
                                .enum_names()
                                .find(|name| {
                                    name.ends_with(&format!("::{}", property_type))
                                        || *name == *property_type
                                })
                                .unwrap_or(property_type);
                            (
                                "string".to_string(),
                                Some(full_name.to_string()),
                                TiledValueExport::String(String::new()),
                            )
                        } else {
                            // It's a class - use ClassDefault (empty object {})
                            (
                                "class".to_string(),
                                Some(property_type.to_string()),
                                TiledValueExport::ClassDefault,
                            )
                        }
                    }
                    TiledTypeKind::Enum { property_type, .. } => {
                        // Enum types are exported as string with propertyType reference
                        (
                            "string".to_string(),
                            Some(property_type.to_string()),
                            TiledValueExport::String(String::new()),
                        )
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
                TiledTypeKind::Class { property_type }
                | TiledTypeKind::Enum { property_type, .. } => {
                    discover_type_recursive(property_type, world, discovered, output);
                }
                _ => {}
            }
        }
        return;
    }

    // Fall back to Bevy reflection
    let app_type_registry = world.resource::<AppTypeRegistry>();
    let registry = app_type_registry.read();

    if let Some(reflect_type) = registry.get_with_type_path(type_path) {
        if let Some(export) = build_reflected_export(reflect_type, &registry) {
            output.push(TiledTypeOrEnumExport::Type(export));

            // Recursively discover reflected field types
            if let TypeInfo::Struct(struct_info) = reflect_type.type_info() {
                for field in struct_info.iter() {
                    let field_type_path = field.type_path();
                    if !is_primitive_type(field_type_path) {
                        discover_type_recursive(field_type_path, world, discovered, output);
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
                    _ => TiledValueExport::String(String::new()),
                };
                (tiled_type, None, default_value)
            } else {
                (
                    "class".to_string(),
                    Some(field_type_path.to_string()),
                    TiledValueExport::ClassDefault,
                )
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
        "bool" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "alloc::string::String" | "&str"
    )
}

/// Map a primitive Rust type path to a Tiled type string.
fn map_primitive_to_tiled(type_path: &str) -> String {
    match type_path {
        "bool" => "bool",
        "i32" | "i64" | "u32" | "u64" => "int",
        "f32" | "f64" => "float",
        _ => "string",
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
