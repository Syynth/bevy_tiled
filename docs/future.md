# Future Work

## Property Deserialization

### Configurable Parse Failure Behavior
Currently, when a property fails to parse, we silently use the default value. Consider adding a resource to configure this behavior:
- `DeserializationBehavior::SilentDefault` (current)
- `DeserializationBehavior::WarnAndDefault`
- `DeserializationBehavior::Error`

### Optional `#[tiled(required)]` Attribute
Allow marking specific fields as required that will error if missing, even when the struct implements Default. Currently not implemented per user preference.
