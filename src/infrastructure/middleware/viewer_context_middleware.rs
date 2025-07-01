// ViewerContext Middleware - Implements Meta's authentic pattern
// Creates ViewerContext from infrastructure and injects into request extensions

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    infrastructure::{
        tao_core::tao_core::TaoOperations,
        viewer::viewer::ViewerContext,
    },
};

/// Authentication information extracted from request
#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub auth_method: Option<String>,
    pub is_authenticated: bool,
}

/// Trait for application state that contains TAO operations
pub trait HasTaoOperations {
    fn get_tao(&self) -> &Arc<dyn TaoOperations>;
}

/// ViewerContext middleware that creates request-scoped viewer context
/// This implements Meta's pattern where business logic only sees ViewerContext
pub async fn viewer_context_middleware<T>(
    State(app_state): State<T>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> 
where
    T: HasTaoOperations + Clone + Send + Sync + 'static,
{
    // Extract authentication information from request headers
    let auth_info = extract_auth_from_request(request.headers())?;
    
    // Create appropriate ViewerContext based on authentication
    let viewer_context = create_viewer_context(auth_info, app_state.get_tao().clone())?;
    
    // Inject ViewerContext into request extensions for handlers
    request.extensions_mut().insert(viewer_context);
    
    // Continue to next handler
    Ok(next.run(request).await)
}

/// Extract authentication information from request headers
/// This would integrate with your actual authentication system
fn extract_auth_from_request(headers: &HeaderMap) -> Result<AuthInfo, StatusCode> {
    // Check for Authorization header
    if let Some(auth_header) = headers.get("authorization") {
        let auth_str = auth_header
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
            
        // Parse different auth methods
        if auth_str.starts_with("Bearer ") {
            let _token = &auth_str[7..];
            // TODO: Validate JWT token and extract user info
            // For now, mock authenticated user
            return Ok(AuthInfo {
                user_id: Some(1001),
                username: Some("authenticated_user".to_string()),
                auth_method: Some("bearer".to_string()),
                is_authenticated: true,
            });
        } else if auth_str.starts_with("System ") {
            // System authentication for internal operations
            return Ok(AuthInfo {
                user_id: None,
                username: Some("system".to_string()),
                auth_method: Some("system".to_string()),
                is_authenticated: true,
            });
        }
    }
    
    // Check for API key header
    if let Some(_api_key) = headers.get("x-api-key") {
        // TODO: Validate API key
        return Ok(AuthInfo {
            user_id: None,
            username: Some("api_client".to_string()),
            auth_method: Some("api_key".to_string()),
            is_authenticated: true,
        });
    }
    
    // No authentication found - create anonymous viewer
    Ok(AuthInfo {
        user_id: None,
        username: None,
        auth_method: None,
        is_authenticated: false,
    })
}

/// Create appropriate ViewerContext based on authentication info
/// This implements Meta's pattern of different viewer types
fn create_viewer_context(
    auth_info: AuthInfo,
    tao: Arc<dyn TaoOperations>,
) -> Result<Arc<ViewerContext>, StatusCode> {
    let request_id = format!("req-{}", Uuid::new_v4());
    
    let viewer_context = match (auth_info.is_authenticated, auth_info.auth_method.as_deref()) {
        // Authenticated user with bearer token
        (true, Some("bearer")) => {
            let user_id = auth_info.user_id.unwrap_or(1001);
            let username = auth_info.username.unwrap_or_else(|| "unknown_user".to_string());
            ViewerContext::authenticated_user(user_id, username, request_id, tao)
        },
        
        // System authentication for internal operations
        (true, Some("system")) => {
            ViewerContext::system(request_id, tao)
        },
        
        // API key authentication (treat as application)
        (true, Some("api_key")) => {
            ViewerContext::system(request_id, tao) // Could create Application type later
        },
        
        // Anonymous/unauthenticated request
        _ => {
            ViewerContext::anonymous(request_id, tao)
        },
    };
    
    Ok(Arc::new(viewer_context))
}

/// Helper to create system viewer context for internal operations
pub fn create_system_viewer_context(tao: Arc<dyn TaoOperations>) -> Arc<ViewerContext> {
    let request_id = format!("system-{}", Uuid::new_v4());
    Arc::new(ViewerContext::system(request_id, tao))
}

/// Helper to create authenticated user viewer context
pub fn create_user_viewer_context(
    user_id: i64,
    username: String,
    tao: Arc<dyn TaoOperations>,
) -> Arc<ViewerContext> {
    let request_id = format!("user-{}-{}", user_id, Uuid::new_v4());
    Arc::new(ViewerContext::authenticated_user(user_id, username, request_id, tao))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    
    #[test]
    fn test_extract_auth_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer token123"));
        
        let auth_info = extract_auth_from_request(&headers).unwrap();
        assert!(auth_info.is_authenticated);
        assert_eq!(auth_info.auth_method, Some("bearer".to_string()));
        assert_eq!(auth_info.user_id, Some(1001));
    }
    
    #[test]
    fn test_extract_auth_system() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("System internal"));
        
        let auth_info = extract_auth_from_request(&headers).unwrap();
        assert!(auth_info.is_authenticated);
        assert_eq!(auth_info.auth_method, Some("system".to_string()));
        assert_eq!(auth_info.username, Some("system".to_string()));
    }
    
    #[test]
    fn test_extract_auth_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("api123"));
        
        let auth_info = extract_auth_from_request(&headers).unwrap();
        assert!(auth_info.is_authenticated);
        assert_eq!(auth_info.auth_method, Some("api_key".to_string()));
    }
    
    #[test]
    fn test_extract_auth_anonymous() {
        let headers = HeaderMap::new();
        
        let auth_info = extract_auth_from_request(&headers).unwrap();
        assert!(!auth_info.is_authenticated);
        assert_eq!(auth_info.auth_method, None);
        assert_eq!(auth_info.user_id, None);
    }
}