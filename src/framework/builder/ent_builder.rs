use crate::framework::entity::ent_trait::Entity;

/// A generic builder trait implemented directly on entity types.
/// This eliminates the need for separate builder structs.
pub trait EntBuilder: Entity + Sized + Send + Sync {
    /// The type that holds the builder state during construction.
    type BuilderState: Default + Send + Sync;

    /// Build the entity with a given ID and builder state.
    /// This method is called by TAO after ID generation.
    fn build(state: Self::BuilderState, id: i64) -> Result<Self, String>;

    /// Returns the type name of the entity.
    fn entity_type() -> &'static str;
}
