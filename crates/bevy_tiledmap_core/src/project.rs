//! Tiled project file (.tiled-project) parsing and property access.
//!
//! Provides access to custom property type definitions from Tiled project files.
//! These definitions include default values for classes and enum variants.
//!
//! # Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_tiledmap_core::{TiledmapCorePlugin, TiledmapCoreConfig};
//! use bevy_tiledmap_core::project::TiledProjectProperties;
//!
//! App::new()
//!     .add_plugins(TiledmapCorePlugin::new(TiledmapCoreConfig {
//!         project_path: Some("assets/my.tiled-project".into()),
//!         ..default()
//!     }));
//!
//! fn use_defaults(props: Res<TiledProjectProperties>) {
//!     // Get a class definition by name
//!     if let Some(class) = props.get_class("avian::PhysicsSettings") {
//!         println!("Found class with {} members", class.members.len());
//!     }
//! }
//! ```

use bevy::prelude::*;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use thiserror::Error;

/// Raw JSON asset loaded from a `.tiled-project` file.
///
/// This asset is loaded automatically via `bevy_common_assets::JsonAssetPlugin`.
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TiledProjectAsset {
    /// Property type definitions (classes and enums).
    #[serde(default)]
    pub property_types: Vec<PropertyType>,

    /// Top-level properties defined in the project.
    #[serde(default)]
    pub properties: Vec<serde_json::Value>,

    /// Automapping rules file path.
    #[serde(default)]
    pub automapping_rules_file: String,

    /// Compatibility version number.
    #[serde(default)]
    pub compatibility_version: u32,

    /// Path to extensions folder.
    #[serde(default)]
    pub extensions_path: String,

    /// Project folders.
    #[serde(default)]
    pub folders: Vec<String>,
}

/// A property type definition from the Tiled project.
///
/// Can be either a class (struct-like) or an enum.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PropertyType {
    /// A class definition with member fields.
    #[serde(rename = "class")]
    Class(ClassDefinition),

    /// An enum definition with named variants.
    #[serde(rename = "enum")]
    Enum(EnumDefinition),
}

impl PropertyType {
    /// Get the name of this property type.
    pub fn name(&self) -> &str {
        match self {
            PropertyType::Class(c) => &c.name,
            PropertyType::Enum(e) => &e.name,
        }
    }

    /// Get the ID of this property type.
    pub fn id(&self) -> u32 {
        match self {
            PropertyType::Class(c) => c.id,
            PropertyType::Enum(e) => e.id,
        }
    }
}

/// A class definition from a Tiled project.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassDefinition {
    /// Unique identifier for this class.
    pub id: u32,

    /// Name of the class (e.g., `"avian::PhysicsSettings"`).
    pub name: String,

    /// Member fields of the class.
    #[serde(default)]
    pub members: Vec<ClassMember>,

    /// Display color in Tiled editor (hex string).
    #[serde(default)]
    pub color: String,

    /// Whether to draw fill in Tiled editor.
    #[serde(default)]
    pub draw_fill: bool,

    /// Where this class can be used (e.g., ["property", "object"]).
    #[serde(default)]
    pub use_as: Vec<String>,
}

/// A member field of a class definition.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassMember {
    /// Name of the member field.
    pub name: String,

    /// Type of the member (e.g., "float", "bool", "string", "int").
    #[serde(rename = "type")]
    pub member_type: String,

    /// Default value for this member.
    pub value: serde_json::Value,

    /// Optional property type name for nested types (e.g., `"avian::BodyType"`).
    #[serde(default)]
    pub property_type: Option<String>,
}

/// An enum definition from a Tiled project.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumDefinition {
    /// Unique identifier for this enum.
    pub id: u32,

    /// Name of the enum (e.g., `"avian::BodyType"`).
    pub name: String,

    /// Enum variant names.
    #[serde(default)]
    pub values: Vec<String>,

    /// How the enum is stored (e.g., "string", "int").
    #[serde(default)]
    pub storage_type: String,

    /// Whether values can be combined as flags.
    #[serde(default)]
    pub values_as_flags: bool,
}

/// Error type for project property deserialization.
#[derive(Debug, Clone, Error)]
pub enum ProjectDeserializeError {
    /// Class not found in project.
    #[error("Class '{0}' not found in Tiled project")]
    ClassNotFound(String),
    /// Serde deserialization failed.
    #[error("Failed to deserialize class: {0}")]
    DeserializeFailed(String),
}

/// Resource providing access to Tiled project property definitions.
///
/// This resource is automatically populated when a project path is configured
/// in `TiledmapCoreConfig`.
#[derive(Resource, Default, Debug)]
pub struct TiledProjectProperties {
    /// Classes indexed by name.
    classes: HashMap<String, ClassDefinition>,

    /// Enums indexed by name.
    enums: HashMap<String, EnumDefinition>,

    /// Project-level property values indexed by name.
    properties: HashMap<String, serde_json::Value>,
}

impl TiledProjectProperties {
    /// Create a new empty properties collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build properties from a loaded project asset.
    pub fn from_asset(asset: &TiledProjectAsset) -> Self {
        let mut classes = HashMap::new();
        let mut enums = HashMap::new();
        let mut properties = HashMap::new();

        for prop_type in &asset.property_types {
            match prop_type {
                PropertyType::Class(class) => {
                    classes.insert(class.name.clone(), class.clone());
                }
                PropertyType::Enum(enum_def) => {
                    enums.insert(enum_def.name.clone(), enum_def.clone());
                }
            }
        }

        // Extract project-level property values
        for prop in &asset.properties {
            if let Some(name) = prop.get("name").and_then(|v| v.as_str())
                && let Some(value) = prop.get("value")
            {
                properties.insert(name.to_string(), value.clone());
            }
        }

        Self {
            classes,
            enums,
            properties,
        }
    }

    /// Get a class definition by name.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(class) = props.get_class("avian::PhysicsSettings") {
    ///     for member in &class.members {
    ///         println!("{}: {:?}", member.name, member.value);
    ///     }
    /// }
    /// ```
    pub fn get_class(&self, name: &str) -> Option<&ClassDefinition> {
        self.classes.get(name)
    }

    /// Get an enum definition by name.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(enum_def) = props.get_enum("avian::BodyType") {
    ///     println!("Variants: {:?}", enum_def.values);
    /// }
    /// ```
    pub fn get_enum(&self, name: &str) -> Option<&EnumDefinition> {
        self.enums.get(name)
    }

    /// Get a class member's default value by class and member name.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(friction) = props.get_member_value("avian::PhysicsSettings", "friction") {
    ///     println!("Default friction: {}", friction);
    /// }
    /// ```
    pub fn get_member_value(
        &self,
        class_name: &str,
        member_name: &str,
    ) -> Option<&serde_json::Value> {
        self.classes
            .get(class_name)?
            .members
            .iter()
            .find(|m| m.name == member_name)
            .map(|m| &m.value)
    }

    /// Iterate over all class definitions.
    pub fn classes(&self) -> impl Iterator<Item = &ClassDefinition> {
        self.classes.values()
    }

    /// Iterate over all enum definitions.
    pub fn enums(&self) -> impl Iterator<Item = &EnumDefinition> {
        self.enums.values()
    }

    /// Check if a class exists.
    pub fn has_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    /// Check if an enum exists.
    pub fn has_enum(&self, name: &str) -> bool {
        self.enums.contains_key(name)
    }

    /// Get a project-level property value by name.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(value) = props.get_property("starting_spawn") {
    ///     println!("Starting spawn: {:?}", value);
    /// }
    /// ```
    pub fn get_property(&self, name: &str) -> Option<&serde_json::Value> {
        self.properties.get(name)
    }

    /// Deserialize a project-level property value into type `T`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct SpawnTarget {
    ///     target_map: String,
    ///     target_tile: IVec2,
    /// }
    ///
    /// fn get_spawn(props: Res<TiledProjectProperties>) {
    ///     match props.get_property_as::<SpawnTarget>("starting_spawn") {
    ///         Ok(spawn) => println!("Starting map: {}", spawn.target_map),
    ///         Err(e) => warn!("No starting spawn: {}", e),
    ///     }
    /// }
    /// ```
    pub fn get_property_as<T: DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<T, ProjectDeserializeError> {
        let value = self.properties.get(name).ok_or_else(|| {
            ProjectDeserializeError::ClassNotFound(format!("property '{}'", name))
        })?;

        serde_json::from_value(value.clone())
            .map_err(|e| ProjectDeserializeError::DeserializeFailed(e.to_string()))
    }

    /// Check if a project-level property exists.
    pub fn has_property(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    /// Deserialize a class definition's default values into type `T`.
    ///
    /// This method looks up a class by name and deserializes its member
    /// default values into a concrete Rust type using serde.
    ///
    /// # Type Requirements
    ///
    /// `T` must implement `serde::de::DeserializeOwned`. Use `#[derive(Deserialize)]`
    /// on your struct to satisfy this.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct PhysicsSettings {
    ///     friction: f32,
    ///     restitution: f32,
    ///     is_sensor: bool,
    /// }
    ///
    /// fn use_defaults(props: Res<TiledProjectProperties>) {
    ///     match props.get_class_as::<PhysicsSettings>("avian::PhysicsSettings") {
    ///         Ok(settings) => println!("Friction: {}", settings.friction),
    ///         Err(e) => warn!("Failed to get settings: {}", e),
    ///     }
    /// }
    /// ```
    pub fn get_class_as<T: DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<T, ProjectDeserializeError> {
        let class = self
            .classes
            .get(name)
            .ok_or_else(|| ProjectDeserializeError::ClassNotFound(name.to_string()))?;

        // Build JSON object from members (name -> value)
        let mut map = serde_json::Map::new();
        for member in &class.members {
            map.insert(member.name.clone(), member.value.clone());
        }

        serde_json::from_value(serde_json::Value::Object(map))
            .map_err(|e| ProjectDeserializeError::DeserializeFailed(e.to_string()))
    }
}
