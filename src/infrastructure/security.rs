// Production-grade Security and Authentication Framework
// Implements enterprise-level security patterns

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use tracing::{info, warn, instrument};
use async_trait::async_trait;

use crate::error::{AppResult, AppError};
use crate::infrastructure::tao::TaoId;

/// Security context for all operations
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: Option<TaoId>,
    pub session_id: String,
    pub permissions: HashSet<Permission>,
    pub rate_limit_tokens: u32,
    pub ip_address: String,
    pub user_agent: String,
    pub authenticated_at: SystemTime,
    pub expires_at: SystemTime,
}

/// Fine-grained permission system
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Permission {
    pub resource: Resource,
    pub action: Action,
    pub scope: Scope,
}

impl Permission {
    pub fn create_object(otype: String) -> Self {
        Permission {
            resource: Resource::System, // Or more specific if schema-aware
            action: Action::Create,
            scope: Scope::Global,
        }
    }

    pub fn update_object(object_id: TaoId) -> Self {
        Permission {
            resource: Resource::User(object_id), // Assuming object_id refers to a user's object
            action: Action::Update,
            scope: Scope::Self_, // Or more specific scope based on context
        }
    }

    pub fn delete_object(object_id: TaoId) -> Self {
        Permission {
            resource: Resource::User(object_id),
            action: Action::Delete,
            scope: Scope::Self_,
        }
    }

    pub fn create_association(id1: TaoId, id2: TaoId, atype: String) -> Self {
        Permission {
            resource: Resource::Association { from: id1, to: id2 },
            action: Action::Create,
            scope: Scope::Self_, // Or more specific scope
        }
    }
    pub fn delete_association(id1: TaoId, id2: TaoId, atype: String) -> Self {
        Permission {
            resource: Resource::Association { from: id1, to: id2 },
            action: Action::Delete,
            scope: Scope::Self_,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Resource {
    User(TaoId),
    Post(TaoId),
    Group(TaoId),
    Page(TaoId),
    Event(TaoId),
    Association { from: TaoId, to: TaoId },
    System,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
    Delete,
    Admin,
    Create,
    Update,
    Share,
    Comment,
    Like,
    Follow,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Scope {
    Self_,           // Only own resources
    Friends,         // Friend's resources
    FriendsOfFriends, // Extended network
    Public,          // Public resources
    Group(TaoId),    // Group-specific resources
    Global,          // System-wide (admin only)
}

/// JWT Claims for authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub iat: u64,          // Issued at
    pub exp: u64,          // Expires at
    pub aud: String,       // Audience
    pub iss: String,       // Issuer
    pub permissions: Vec<String>, // Encoded permissions
    pub session_id: String,
    pub role: String,
}

/// User credentials and profile
#[derive(Debug, Clone)]
pub struct UserCredentials {
    pub user_id: TaoId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: SystemTime,
    pub last_login: Option<SystemTime>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    User,
    Moderator,
    Admin,
    System,
}

impl UserRole {
    pub fn get_default_permissions(&self) -> HashSet<Permission> {
        match self {
            UserRole::User => {
                let mut perms = HashSet::new();
                perms.insert(Permission {
                    resource: Resource::System,
                    action: Action::Read,
                    scope: Scope::Public,
                });
                perms
            }
            UserRole::Moderator => {
                let mut perms = UserRole::User.get_default_permissions();
                perms.insert(Permission {
                    resource: Resource::System,
                    action: Action::Admin,
                    scope: Scope::Public,
                });
                perms
            }
            UserRole::Admin => {
                let mut perms = HashSet::new();
                perms.insert(Permission {
                    resource: Resource::System,
                    action: Action::Admin,
                    scope: Scope::Global,
                });
                perms
            }
            UserRole::System => {
                let mut perms = HashSet::new();
                perms.insert(Permission {
                    resource: Resource::System,
                    action: Action::Admin,
                    scope: Scope::Global,
                });
                perms
            }
        }
    }
}

/// Authentication and authorization service
pub struct SecurityService {
    /// JWT signing keys
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,

    /// User credentials store (in production, this would be a database)
    credentials: Arc<RwLock<HashMap<String, UserCredentials>>>,

    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, SecurityContext>>>,

    /// Rate limiting state
    rate_limiter: Arc<RateLimiter>,

    /// Audit logger
    audit_logger: Arc<AuditLogger>,

    /// Security configuration
    config: SecurityConfig,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiry: Duration,
    pub password_min_length: usize,
    pub max_failed_attempts: u32,
    pub lockout_duration: Duration,
    pub session_timeout: Duration,
    pub require_2fa: bool,
    pub rate_limit_requests_per_minute: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key".to_string(), // In production, from env
            jwt_expiry: Duration::from_secs(24 * 3600),
            password_min_length: 8,
            max_failed_attempts: 5,
            lockout_duration: Duration::from_secs(900), // 15 minutes
            session_timeout: Duration::from_secs(8 * 3600),
            require_2fa: false,
            rate_limit_requests_per_minute: 60,
        }
    }
}

impl SecurityService {
    pub fn new(config: SecurityConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());

        Self {
            encoding_key,
            decoding_key,
            credentials: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RateLimiter::new(config.rate_limit_requests_per_minute)),
            audit_logger: Arc::new(AuditLogger::new()),
            config,
        }
    }

    /// Register a new user with secure password hashing
    #[instrument(skip(self, password))]
    pub async fn register_user(&self, username: String, email: String, password: String) -> AppResult<TaoId> {
        // Validate password strength
        if password.len() < self.config.password_min_length {
            return Err(AppError::Validation(format!("Password must be at least {} characters", self.config.password_min_length)));
        }

        // Check if user already exists
        {
            let credentials = self.credentials.read().await;
            if credentials.values().any(|cred| cred.username == username || cred.email == email) {
                return Err(AppError::Validation("User already exists".to_string()));
            }
        }

        // Hash password securely
        let password_hash = self.hash_password(&password)?;

        // Generate new user ID
        let user_id = crate::infrastructure::id_generator::get_id_generator().next_id();

        let user_credentials = UserCredentials {
            user_id,
            username: username.clone(),
            email: email.clone(),
            password_hash,
            role: UserRole::User,
            is_active: true,
            created_at: SystemTime::now(),
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
        };

        // Store credentials
        {
            let mut credentials = self.credentials.write().await;
            credentials.insert(username.clone(), user_credentials);
        }

        // Audit log
        self.audit_logger.log_event(AuditEvent {
            event_type: AuditEventType::UserRegistered,
            user_id: Some(user_id),
            ip_address: "unknown".to_string(),
            timestamp: SystemTime::now(),
            details: format!("User {} registered with email {}", username, email),
        }).await;

        info!("User {} registered successfully", username);
        Ok(user_id)
    }

    /// Authenticate user and create session
    #[instrument(skip(self, password))]
    pub async fn authenticate(&self, username: String, password: String, ip_address: String) -> AppResult<String> {
        // Rate limiting check
        if !self.rate_limiter.check_rate_limit(&ip_address).await {
            self.audit_logger.log_event(AuditEvent {
                event_type: AuditEventType::RateLimitExceeded,
                user_id: None,
                ip_address: ip_address.clone(),
                timestamp: SystemTime::now(),
                details: "Authentication rate limit exceeded".to_string(),
            }).await;
            return Err(AppError::TooManyRequests("Rate limit exceeded".to_string()));
        }

        let mut user_creds = {
            let credentials = self.credentials.read().await;
            credentials.get(&username)
                .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?
                .clone()
        };

        // Check if account is locked
        if let Some(locked_until) = user_creds.locked_until {
            if SystemTime::now() < locked_until {
                return Err(AppError::Forbidden("Account is locked".to_string()));
            } else {
                // Unlock account
                user_creds.locked_until = None;
                user_creds.failed_login_attempts = 0;
            }
        }

        // Verify password
        if !self.verify_password(&password, &user_creds.password_hash)? {
            // Increment failed attempts
            user_creds.failed_login_attempts += 1;

            // Lock account if too many failures
            if user_creds.failed_login_attempts >= self.config.max_failed_attempts {
                user_creds.locked_until = Some(SystemTime::now() + self.config.lockout_duration);
                warn!("Account {} locked due to too many failed attempts", username);
            }

            // Save user_id before moving user_creds
            let user_id = user_creds.user_id;

            // Update credentials
            {
                let mut credentials = self.credentials.write().await;
                credentials.insert(username.clone(), user_creds);
            }

            self.audit_logger.log_event(AuditEvent {
                event_type: AuditEventType::LoginFailed,
                user_id: Some(user_id),
                ip_address: ip_address.clone(),
                timestamp: SystemTime::now(),
                details: "Invalid password".to_string(),
            }).await;

            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        // Reset failed attempts on successful login
        user_creds.failed_login_attempts = 0;
        user_creds.last_login = Some(SystemTime::now());

        // Update credentials
        {
            let mut credentials = self.credentials.write().await;
            credentials.insert(username.clone(), user_creds.clone());
        }

        // Create JWT token
        let session_id = uuid::Uuid::new_v4().to_string();
        let token = self.create_jwt_token(&user_creds, &session_id)?;

        // Create security context
        let security_context = SecurityContext {
            user_id: Some(user_creds.user_id),
            session_id: session_id.clone(),
            permissions: user_creds.role.get_default_permissions(),
            rate_limit_tokens: self.config.rate_limit_requests_per_minute,
            ip_address: ip_address.clone(),
            user_agent: "unknown".to_string(),
            authenticated_at: SystemTime::now(),
            expires_at: SystemTime::now() + self.config.session_timeout,
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, security_context);
        }

        self.audit_logger.log_event(AuditEvent {
            event_type: AuditEventType::LoginSuccessful,
            user_id: Some(user_creds.user_id),
            ip_address,
            timestamp: SystemTime::now(),
            details: "User authenticated successfully".to_string(),
        }).await;

        info!("User {} authenticated successfully", username);
        Ok(token)
    }

    /// Check if user has permission for a specific action
    #[instrument(skip(self))]
    pub async fn check_permission(&self, context: &SecurityContext, permission: &Permission) -> bool {
        // This is the actual implementation of permission checking
        // The trait implementation will just delegate to this.
        self.check_permission_internal(context, permission).await
    }

    /// Validate JWT token and return security context
    #[instrument(skip(self, token))]
    pub async fn validate_token(&self, token: &str) -> AppResult<SecurityContext> {
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

        let claims = token_data.claims;

        // Check if session exists and is valid
        let sessions = self.sessions.read().await;
        let context = sessions.get(&claims.session_id)
            .ok_or_else(|| AppError::Unauthorized("Session not found".to_string()))?;

        // Check if session has expired
        if SystemTime::now() > context.expires_at {
            return Err(AppError::Unauthorized("Session expired".to_string()));
        }

        Ok(context.clone())
    }

    /// Logout and invalidate session
    #[instrument(skip(self))]
    pub async fn logout(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(context) = sessions.remove(session_id) {
            self.audit_logger.log_event(AuditEvent {
                event_type: AuditEventType::LogoutSuccessful,
                user_id: context.user_id,
                ip_address: context.ip_address,
                timestamp: SystemTime::now(),
                details: "User logged out".to_string(),
            }).await;
        }

        Ok(())
    }

    /// Hash password securely using Argon2
    fn hash_password(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify password against hash
    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;

        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Create JWT token
    fn create_jwt_token(&self, user_creds: &UserCredentials, session_id: &str) -> AppResult<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let exp = now + self.config.jwt_expiry.as_secs();

        let claims = Claims {
            sub: user_creds.user_id.to_string(),
            iat: now,
            exp,
            aud: "tao-database".to_string(),
            iss: "tao-auth-service".to_string(),
            permissions: vec![], // Simplified for now
            session_id: session_id.to_string(),
            role: format!("{:?}", user_creds.role),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
    }

    /// Internal permission checking logic
    async fn check_permission_internal(&self, context: &SecurityContext, permission: &Permission) -> bool {
        // System administrators have all permissions
        if context.permissions.contains(&Permission {
            resource: Resource::System,
            action: Action::Admin,
            scope: Scope::Global,
        }) {
            return true;
        }

        // Check exact permission match
        if context.permissions.contains(permission) {
            return true;
        }

        // Check scope-based permissions
        match &permission.resource {
            Resource::User(user_id) => {
                if context.user_id == Some(*user_id) { return true; }
                self.check_scope_permission(context, permission).await
            }
            Resource::Association { from, to } => {
                if context.user_id == Some(*from) || context.user_id == Some(*to) { return true; }
                false
            }
            _ => self.check_scope_permission(context, permission).await,
        }
    }

    /// Check scope-based permissions (simplified - placeholder for complex logic)
    async fn check_scope_permission(&self, _context: &SecurityContext, _permission: &Permission) -> bool {
        // In production, this would check friend relationships, group memberships, etc.
        false
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = SystemTime::now();

        sessions.retain(|_, context| context.expires_at > now);
    }
}

/// Rate limiter for API requests
#[derive(Debug)]
pub struct RateLimiter {
    requests_per_minute: u32,
    windows: Arc<RwLock<HashMap<String, RateLimitWindow>>>,
}

#[derive(Debug)]
struct RateLimitWindow {
    requests: u32,
    window_start: SystemTime,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            windows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, identifier: &str) -> bool {
        let mut windows = self.windows.write().await;
        let now = SystemTime::now();

        let window = windows.entry(identifier.to_string()).or_insert(RateLimitWindow {
            requests: 0,
            window_start: now,
        });

        // Reset window if a minute has passed
        if now.duration_since(window.window_start).unwrap_or_default() >= Duration::from_secs(60) {
            window.requests = 0;
            window.window_start = now;
        }

        if window.requests >= self.requests_per_minute {
            false
        } else {
            window.requests += 1;
            true
        }
    }
}

/// Audit logging for security events
#[derive(Debug)]
pub struct AuditLogger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub user_id: Option<TaoId>,
    pub ip_address: String,
    pub timestamp: SystemTime,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum AuditEventType {
    UserRegistered,
    LoginSuccessful,
    LoginFailed,
    LogoutSuccessful,
    PermissionDenied,
    RateLimitExceeded,
    DataAccessed,
    DataModified,
    SecurityViolation,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn log_event(&self, event: AuditEvent) {
        let mut events = self.events.write().await;
        events.push(event.clone());

        // In production, this would write to a secure audit log
        info!("AUDIT: {:?} - {} - {:?}", event.event_type, event.details, event.user_id);
    }

    pub async fn get_events_for_user(&self, user_id: TaoId) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|event| event.user_id == Some(user_id))
            .cloned()
            .collect()
    }
}

/// Security middleware for request processing
pub struct SecurityMiddleware {
    security_service: Arc<SecurityService>,
}

impl SecurityMiddleware {
    pub fn new(security_service: Arc<SecurityService>) -> Self {
        Self { security_service }
    }

    /// Extract and validate security context from request
    pub async fn extract_context(&self, authorization_header: Option<&str>) -> AppResult<SecurityContext> {
        let token = authorization_header
            .ok_or_else(|| AppError::Unauthorized("Authorization header required".to_string()))?
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".to_string()))?;

        self.security_service.validate_token(token).await
    }

    /// Require specific permission for operation
    pub async fn require_permission(&self, context: &SecurityContext, permission: Permission) -> AppResult<()> {
        if !self.security_service.check_permission(context, &permission).await {
            return Err(AppError::Forbidden("Insufficient permissions".to_string()));
        }
        Ok(())
    }
}

#[async_trait]
impl crate::infrastructure::traits::SecurityInterface for SecurityService {
    async fn check_permission(&self, context: &SecurityContext, permission: &Permission) -> bool {
        self.check_permission_internal(context, permission).await
    }
}