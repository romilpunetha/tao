// ViewerContext Extractor - Clean, ergonomic API for handlers
// Provides transparent Arc management with reference-like ergonomics

use std::sync::Arc;
use crate::infrastructure::viewer::viewer::ViewerContext;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

/// Ergonomic ViewerContext wrapper that provides reference-like semantics
/// while maintaining thread-safe Arc behavior for async functions.
/// 
/// ## Key Benefits:
/// - **Thread Safe**: Uses Arc internally, safe across async await points
/// - **Reference Feel**: Implements Clone, so `vc` can be passed around easily
/// - **Zero Overhead**: Clone is just cloning an Arc pointer (very fast)
/// - **Clean API**: No need for explicit .clone_arc() calls in most cases
///
/// ## Usage:
/// ```rust
/// async fn handler(vc: Vc, Json(data): Json<RequestData>) -> impl IntoResponse {
///     // Access fields directly - feels like a reference
///     println!("User: {:?}", vc.user_id);
///     
///     // Pass to entity operations - just pass vc directly!
///     let user = EntUser::create(vc).username("test").savex().await?;
///     
///     // Can use vc multiple times without cloning
///     let posts = EntPost::gen_all(vc).await?;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Vc(Arc<ViewerContext>);

impl Vc {
    /// Create a new Vc wrapper from Arc<ViewerContext>
    pub fn new(vc: Arc<ViewerContext>) -> Self {
        Self(vc)
    }
    
    /// Get the inner Arc<ViewerContext> (rarely needed)
    pub fn arc(self) -> Arc<ViewerContext> {
        self.0
    }
}

// Implement Deref so you can access ViewerContext fields directly
// Examples: vc.user_id, vc.request_id, vc.is_authenticated(), etc.
impl std::ops::Deref for Vc {
    type Target = ViewerContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Implement From so entity operations can accept Vc directly
impl From<Vc> for Arc<ViewerContext> {
    fn from(vc: Vc) -> Self {
        vc.0
    }
}

// Implement From for &Vc so entity operations can accept references too
impl From<&Vc> for Arc<ViewerContext> {
    fn from(vc: &Vc) -> Self {
        vc.0.clone()
    }
}

// Auto-convert Vc to Arc<ViewerContext> for entity operations
impl AsRef<ViewerContext> for Vc {
    fn as_ref(&self) -> &ViewerContext {
        &self.0
    }
}

// Implement FromRequestParts so Axum can extract Vc from request extensions
impl<S> FromRequestParts<S> for Vc
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let vc = parts
            .extensions
            .get::<Arc<ViewerContext>>()
            .map(|vc| Vc(vc.clone()))
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        async move { vc }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::{
        tao_core::tao_core::TaoOperations,
        viewer::viewer::ViewerContext,
    };
    use std::sync::Arc;

    // Mock TaoOperations for testing
    struct MockTao;
    
    #[async_trait::async_trait]
    impl TaoOperations for MockTao {
        async fn get_object(&self, _id: i64) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(None)
        }
        
        async fn create_object(&self, _data: Vec<u8>) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            Ok(1)
        }
        
        // ... other required methods would be implemented for a real test
        async fn update_object(&self, _id: i64, _data: Vec<u8>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn delete_object(&self, _id: i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(true)
        }
        
        async fn assoc_add(&self, _assoc: crate::infrastructure::tao_core::tao_core::TaoAssociation) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn assoc_get(&self, _query: crate::infrastructure::tao_core::tao_core::TaoAssocQuery) -> Result<Vec<crate::infrastructure::tao_core::tao_core::TaoAssociation>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![])
        }
        
        async fn assoc_delete(&self, _query: crate::infrastructure::tao_core::tao_core::TaoAssocQuery) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
            Ok(0)
        }
        
        async fn assoc_count(&self, _query: crate::infrastructure::tao_core::tao_core::TaoAssocQuery) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
            Ok(0)
        }
        
        async fn create_entity<T>(&self, _builder_state: T) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
        where
            T: Send + Sync,
        {
            todo!("Mock implementation")
        }
    }

    #[test]
    fn test_vc_deref() {
        let mock_tao: Arc<dyn TaoOperations> = Arc::new(MockTao);
        let viewer_context = Arc::new(ViewerContext::system(
            "test-request".to_string(),
            mock_tao,
        ));
        let vc = Vc(viewer_context.clone());
        
        // Test that we can access ViewerContext fields directly
        assert_eq!(vc.request_id, "test-request");
        
        // Test that get() returns a reference
        let vc_ref = vc.get();
        assert_eq!(vc_ref.request_id, "test-request");
        
        // Test that arc() returns the Arc
        let vc_arc = vc.arc();
        assert_eq!(vc_arc.request_id, "test-request");
    }
}