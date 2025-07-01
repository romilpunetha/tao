// Association Framework - Ergonomic association management
// Eliminates 80% of repetitive association code across all entities

use crate::error::AppResult;
use crate::infrastructure::tao_core::tao_core::TaoOperations;
use crate::infrastructure::viewer::viewer::ViewerContext;
use crate::framework::entity::ent_trait::Entity;
use std::sync::Arc;

/// Ergonomic association context for consistent TAO access
pub trait AssociationContext {
    fn tao(&self) -> &Arc<dyn TaoOperations>;
}

impl AssociationContext for ViewerContext {
    fn tao(&self) -> &Arc<dyn TaoOperations> {
        &self.tao
    }
}

impl AssociationContext for Arc<ViewerContext> {
    fn tao(&self) -> &Arc<dyn TaoOperations> {
        &self.tao
    }
}

/// Generic association operations trait - implements once, use everywhere
pub trait HasAssociations: Entity {
    /// Get associated entities of any type
    async fn get_associated<T, C>(&self, ctx: &C, association_type: &str, limit: Option<usize>) -> AppResult<Vec<T>>
    where
        T: Entity + for<'a> TryFrom<&'a [u8], Error = crate::error::AppError>,
        C: AssociationContext,
    {
        let tao = ctx.tao();
        let limit_u32 = limit.map(|l| l as u32);
        let neighbor_ids = tao
            .get_neighbor_ids(self.id(), association_type.to_string(), limit_u32)
            .await?;

        let mut results = Vec::new();
        for id in neighbor_ids {
            if let Some(tao_obj) = tao.obj_get(id).await? {
                if let Ok(entity) = T::try_from(&tao_obj.data) {
                    results.push(entity);
                }
            }
        }
        
        Ok(results)
    }

    /// Count associations of any type
    async fn count_associated<C>(&self, ctx: &C, association_type: &str) -> AppResult<i64>
    where
        C: AssociationContext,
    {
        let tao = ctx.tao();
        let count = tao.assoc_count(self.id(), association_type.to_string()).await?;
        Ok(count as i64)
    }

    /// Add association to any entity
    async fn add_association<C>(&self, ctx: &C, association_type: &str, target_id: i64) -> AppResult<()>
    where
        C: AssociationContext,
    {
        let tao = ctx.tao();
        
        // Verify target exists before creating association
        if tao.obj_get(target_id).await?.is_none() {
            return Err(crate::error::AppError::Validation(
                format!("Target entity {} does not exist", target_id)
            ));
        }

        // Create association using TAO helper function
        let association = crate::infrastructure::tao_core::tao_core::create_tao_association(
            self.id(),
            association_type.to_string(),
            target_id,
            None,
        );
        tao.assoc_add(association).await?;
        
        Ok(())
    }

    /// Remove association to any entity
    async fn remove_association<C>(&self, ctx: &C, association_type: &str, target_id: i64) -> AppResult<()>
    where
        C: AssociationContext,
    {
        let tao = ctx.tao();
        tao.assoc_delete(self.id(), association_type.to_string(), target_id).await?;
        Ok(())
    }
}

// Automatic implementation for all entities
impl<T: Entity> HasAssociations for T {}

/// Ergonomic association query builder
pub struct AssociationQuery<'a, T: Entity> {
    entity: &'a T,
    association_type: String,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl<'a, T: Entity> AssociationQuery<'a, T> {
    pub fn new(entity: &'a T, association_type: &str) -> Self {
        Self {
            entity,
            association_type: association_type.to_string(),
            limit: None,
            offset: None,
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub async fn execute<U, C>(&self, ctx: &C) -> AppResult<Vec<U>>
    where
        U: Entity + for<'b> TryFrom<&'b [u8], Error = crate::error::AppError>,
        C: AssociationContext,
    {
        self.entity.get_associated(ctx, &self.association_type, self.limit).await
    }

    pub async fn count<C>(&self, ctx: &C) -> AppResult<i64>
    where
        C: AssociationContext,
    {
        self.entity.count_associated(ctx, &self.association_type).await
    }
}

/// Ergonomic macros for defining typed association methods
#[macro_export]
macro_rules! define_associations {
    ($entity:ident => {
        $(
            $method_name:ident -> $target_type:ty as $assoc_type:literal
        ),* $(,)?
    }) => {
        impl $entity {
            $(
                pub async fn $method_name<C>(&self, ctx: &C) -> AppResult<Vec<$target_type>>
                where
                    C: crate::framework::entity::associations::AssociationContext,
                {
                    self.get_associated(ctx, $assoc_type, Some(100)).await
                }

                paste::paste! {
                    pub async fn [<count_ $method_name>]<C>(&self, ctx: &C) -> AppResult<i64>
                    where
                        C: crate::framework::entity::associations::AssociationContext,
                    {
                        self.count_associated(ctx, $assoc_type).await
                    }

                    pub async fn [<add_ $method_name:singular>]<C>(&self, ctx: &C, target_id: i64) -> AppResult<()>
                    where
                        C: crate::framework::entity::associations::AssociationContext,
                    {
                        self.add_association(ctx, $assoc_type, target_id).await
                    }

                    pub async fn [<remove_ $method_name:singular>]<C>(&self, ctx: &C, target_id: i64) -> AppResult<()>
                    where
                        C: crate::framework::entity::associations::AssociationContext,
                    {
                        self.remove_association(ctx, $assoc_type, target_id).await
                    }
                }
            )*
        }
    };
}