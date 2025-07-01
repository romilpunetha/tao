// Meta-style Viewer Context - Represents the authenticated actor making requests
// Contains all authentication, authorization, and request metadata needed for context-aware operations

use crate::infrastructure::tao_core::tao_core::TaoOperations;
use serde_json::Value;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::SystemTime;

/// Represents different types of actors that can make requests
#[derive(Debug, Clone, PartialEq)]
pub enum ViewerType {
    User,           // Authenticated user
    Application,    // Server-to-server app
    Service,        // Internal service
    Anonymous,      // Unauthenticated request
    System,         // System/admin operation
}

/// Capability types that can be granted to viewers
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Capability {
    // User operations
    CreateUser,
    UpdateOwnProfile,
    UpdateAnyProfile,
    DeleteOwnAccount,
    
    // Content operations
    CreatePost,
    UpdateOwnPost,
    UpdateAnyPost,
    DeleteOwnPost,
    DeleteAnyPost,
    ViewPrivateContent,
    
    // Administrative capabilities
    AdminAccess,
    ModerateContent,
    ManageUsers,
    ViewAnalytics,
    
    // Rate limiting exemptions
    BypassRateLimit,
    HighVolumeOperations,
    
    // Custom capability
    Custom(String),
}

/// Authentication information about the viewer
#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub is_authenticated: bool,
    pub auth_method: Option<String>,    // "password", "oauth", "token", etc.
    pub session_id: Option<String>,
    pub auth_timestamp: Option<SystemTime>,
    pub token_expires_at: Option<SystemTime>,
}

/// Request metadata for audit and security
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub app_id: Option<String>,         // Which app/client is making the request
    pub request_id: String,             // Unique request identifier for tracing
    pub timestamp: SystemTime,
}

/// Privacy settings and preferences
#[derive(Debug, Clone)]
pub struct PrivacySettings {
    pub default_visibility: String,     // "public", "friends", "private"
    pub location_sharing: bool,
    pub analytics_opt_out: bool,
    pub targeted_ads_opt_out: bool,
}

/// Comprehensive viewer context following Meta's architecture
/// Contains all dependencies including database access (TAO)
#[derive(Debug, Clone)]
pub struct ViewerContext {
    // Core identity
    pub viewer_type: ViewerType,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    
    // Authentication & authorization
    pub auth_info: AuthInfo,
    pub roles: Vec<String>,              // "admin", "moderator", "premium_user", etc.
    pub capabilities: Vec<Capability>,   // Granular permissions
    
    // Privacy & personalization
    pub privacy_settings: Option<PrivacySettings>,
    
    // Request context
    pub request_metadata: RequestMetadata,
    
    // Database access - following Meta's pattern where viewer context contains all dependencies
    pub tao: Arc<dyn TaoOperations>,
    
    // Custom metadata for extensibility
    pub custom_data: HashMap<String, Value>,
}

impl ViewerContext {
    /// Create a new authenticated user viewer
    pub fn authenticated_user(
        user_id: i64,
        username: String,
        request_id: String,
        tao: Arc<dyn TaoOperations>,
    ) -> Self {
        Self {
            viewer_type: ViewerType::User,
            user_id: Some(user_id),
            username: Some(username),
            auth_info: AuthInfo {
                is_authenticated: true,
                auth_method: Some("session".to_string()),
                session_id: None,
                auth_timestamp: Some(SystemTime::now()),
                token_expires_at: None,
            },
            roles: vec!["user".to_string()],
            capabilities: vec![
                Capability::CreatePost,
                Capability::UpdateOwnProfile,
                Capability::UpdateOwnPost,
                Capability::DeleteOwnPost,
                Capability::DeleteOwnAccount,
            ],
            privacy_settings: Some(PrivacySettings::default()),
            request_metadata: RequestMetadata {
                ip_address: None,
                user_agent: None,
                locale: Some("en_US".to_string()),
                timezone: None,
                app_id: None,
                request_id,
                timestamp: SystemTime::now(),
            },
            tao,
            custom_data: HashMap::new(),
        }
    }
    
    /// Create an anonymous (unauthenticated) viewer
    pub fn anonymous(request_id: String, tao: Arc<dyn TaoOperations>) -> Self {
        Self {
            viewer_type: ViewerType::Anonymous,
            user_id: None,
            username: None,
            auth_info: AuthInfo {
                is_authenticated: false,
                auth_method: None,
                session_id: None,
                auth_timestamp: None,
                token_expires_at: None,
            },
            roles: vec!["anonymous".to_string()],
            capabilities: vec![], // Very limited capabilities
            privacy_settings: None,
            request_metadata: RequestMetadata {
                ip_address: None,
                user_agent: None,
                locale: Some("en_US".to_string()),
                timezone: None,
                app_id: None,
                request_id,
                timestamp: SystemTime::now(),
            },
            tao,
            custom_data: HashMap::new(),
        }
    }
    
    /// Create a system/admin viewer for internal operations
    pub fn system(request_id: String, tao: Arc<dyn TaoOperations>) -> Self {
        Self {
            viewer_type: ViewerType::System,
            user_id: None,
            username: Some("system".to_string()),
            auth_info: AuthInfo {
                is_authenticated: true,
                auth_method: Some("internal".to_string()),
                session_id: None,
                auth_timestamp: Some(SystemTime::now()),
                token_expires_at: None,
            },
            roles: vec!["system".to_string(), "admin".to_string()],
            capabilities: vec![
                Capability::AdminAccess,
                Capability::ManageUsers,
                Capability::ModerateContent,
                Capability::ViewAnalytics,
                Capability::BypassRateLimit,
                Capability::HighVolumeOperations,
                Capability::UpdateAnyProfile,
                Capability::UpdateAnyPost,
                Capability::DeleteAnyPost,
                Capability::ViewPrivateContent,
            ],
            privacy_settings: None,
            request_metadata: RequestMetadata {
                ip_address: None,
                user_agent: Some("system/1.0".to_string()),
                locale: Some("en_US".to_string()),
                timezone: None,
                app_id: Some("system".to_string()),
                request_id,
                timestamp: SystemTime::now(),
            },
            tao,
            custom_data: HashMap::new(),
        }
    }
    
    /// Check if viewer has a specific capability
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
    
    /// Check if viewer has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
    
    /// Check if viewer is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_info.is_authenticated
    }
    
    /// Check if viewer is an admin
    pub fn is_admin(&self) -> bool {
        self.has_role("admin") || self.has_capability(&Capability::AdminAccess)
    }
    
    /// Check if viewer is the system
    pub fn is_system(&self) -> bool {
        self.viewer_type == ViewerType::System
    }
    
    /// Check if viewer owns a resource (by user_id)
    pub fn owns_resource(&self, owner_id: i64) -> bool {
        self.user_id.map_or(false, |uid| uid == owner_id)
    }
    
    /// Add custom metadata
    pub fn with_custom_data(mut self, key: String, value: Value) -> Self {
        self.custom_data.insert(key, value);
        self
    }
    
    /// Set IP address
    pub fn with_ip_address(mut self, ip: IpAddr) -> Self {
        self.request_metadata.ip_address = Some(ip);
        self
    }
    
    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.request_metadata.user_agent = Some(user_agent);
        self
    }
    
    /// Set app ID
    pub fn with_app_id(mut self, app_id: String) -> Self {
        self.request_metadata.app_id = Some(app_id);
        self
    }
    
    /// Grant additional capability
    pub fn with_capability(mut self, capability: Capability) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
        self
    }
    
    /// Add role
    pub fn with_role(mut self, role: String) -> Self {
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
        self
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            default_visibility: "friends".to_string(),
            location_sharing: false,
            analytics_opt_out: false,
            targeted_ads_opt_out: false,
        }
    }
}

// Conversion to PrivacyContext for compatibility with existing privacy system
impl From<&ViewerContext> for crate::framework::ent_privacy::PrivacyContext {
    fn from(viewer: &ViewerContext) -> Self {
        Self {
            entity_type: crate::framework::schema::ent_schema::EntityType::EntUser, // Will be set by caller
            entity_id: None, // Will be set by caller
            operation: crate::framework::ent_privacy::PrivacyOperation::Read, // Will be set by caller
            user_id: viewer.user_id,
            user_roles: viewer.roles.clone(),
            data: None, // Will be set by caller
            metadata: viewer.custom_data.clone(),
        }
    }
}
