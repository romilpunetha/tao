use crate::ent_framework::Entity;
use async_trait::async_trait;

/// A generic builder trait that all entity builders will implement.
/// This allows the TAO layer to handle the creation process generically.
#[async_trait]
pub trait EntBuilder: Sized + Send {
    /// The type of entity that this builder creates.
    type EntityType: Entity;

    /// Build the entity with a given ID.
    /// This method is responsible for constructing the entity object.
    fn build(self, id: i64) -> Result<Self::EntityType, String>;

    /// Returns the type name of the entity.
    fn entity_type() -> &'static str;
}
