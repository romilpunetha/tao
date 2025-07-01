// Viewer Context - Request-scoped dependency injection following Meta's pattern
// ViewerContext contains all dependencies including TAO operations

use crate::infrastructure::viewer::viewer::ViewerContext;
use crate::infrastructure::tao_core::tao_core::TaoOperations;
use std::future::Future;
use std::sync::Arc;
use tokio::task_local;
use uuid::Uuid;

// Task-local storage for viewer context - contains all dependencies following Meta's pattern
task_local! {
    static VIEWER_CONTEXT: Arc<ViewerContext>;
}

/// Get the viewer context from the current async task scope
pub fn get_viewer_context() -> Result<Arc<ViewerContext>, String> {
    VIEWER_CONTEXT.try_with(|viewer| Arc::clone(viewer))
        .map_err(|_| "Viewer context not set - ensure viewer context is injected at request boundary".to_string())
}

/// Get the TAO context from viewer context (for backward compatibility)
pub fn get_tao_context() -> Result<Arc<dyn TaoOperations>, String> {
    get_viewer_context().map(|vc| Arc::clone(&vc.tao))
}

/// Execute a closure with viewer context set (Meta's pattern)
pub async fn with_viewer_context<F, R>(viewer: Arc<ViewerContext>, f: F) -> R
where
    F: Future<Output = R>,
    R: Send + 'static,
{
    VIEWER_CONTEXT.scope(viewer, f).await
}

/// Backward compatibility: Execute with TAO context 
/// (internally creates a system viewer context)
pub async fn with_tao_context<F, R>(tao: Arc<dyn TaoOperations>, f: F) -> R
where
    F: Future<Output = R>,
    R: Send + 'static,
{
    let viewer = Arc::new(ViewerContext::system(
        format!("tao-compat-{}", Uuid::new_v4()),
        tao
    ));
    with_viewer_context(viewer, f).await
}