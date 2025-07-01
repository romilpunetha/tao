// Ergonomic Builder Enhancements - Improves builder API usability
// Addresses string handling, context passing, and method chaining ergonomics

use crate::error::AppResult;
use crate::infrastructure::viewer::viewer::ViewerContext;
use crate::framework::entity::ent_trait::Entity;
use std::sync::Arc;

/// Enhanced builder context that provides ergonomic API patterns
pub trait ErgonomicBuilder: Sized {
    type Entity: Entity;
    
    /// Accept string slices instead of owned strings for better ergonomics
    fn with_string<S: Into<String>>(mut self, field_setter: impl FnOnce(&mut Self, String), value: S) -> Self {
        field_setter(&mut self, value.into());
        self
    }
    
    /// Conditional field setting for optional fields
    fn with_optional<T>(mut self, field_setter: impl FnOnce(&mut Self, T), value: Option<T>) -> Self {
        if let Some(val) = value {
            field_setter(&mut self, val);
        }
        self
    }
    
    /// Chain multiple field setters
    fn chain<F>(self, f: F) -> Self 
    where 
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
}

/// Builder validation trait for compile-time field checking
pub trait ValidatedBuilder {
    type Error;
    
    /// Validate all required fields are set before building
    fn validate(&self) -> Result<(), Self::Error>;
    
    /// Build only if validation passes
    fn build_validated(self) -> Result<Self, Self::Error> 
    where
        Self: Sized,
    {
        self.validate()?;
        Ok(self)
    }
}

/// Context-aware builder trait for better TAO integration
pub trait ContextualBuilder<C> {
    type Output;
    
    /// Create entity with provided context
    async fn create_with(self, context: C) -> AppResult<Self::Output>;
    
    /// Create entity and associate it with another entity
    async fn create_and_associate_with<E: Entity>(
        self, 
        context: C, 
        parent: &E, 
        association_type: &str
    ) -> AppResult<Self::Output>;
}

/// Macro to enhance existing builder implementations with ergonomic methods
#[macro_export]
macro_rules! enhance_builder {
    ($builder:ident) => {
        impl $builder {
            /// Enhanced username setter accepting string slices
            pub fn username_str<S: Into<String>>(mut self, username: S) -> Self {
                self.username = Some(username.into());
                self
            }
            
            /// Enhanced email setter accepting string slices  
            pub fn email_str<S: Into<String>>(mut self, email: S) -> Self {
                self.email = Some(email.into());
                self
            }
            
            /// Enhanced full_name setter accepting string slices
            pub fn full_name_str<S: Into<String>>(mut self, full_name: S) -> Self {
                self.full_name = Some(full_name.into());
                self
            }
            
            /// Enhanced bio setter accepting string slices
            pub fn bio_str<S: Into<String>>(mut self, bio: S) -> Self {
                self.bio = Some(bio.into());
                self
            }
            
            /// Enhanced location setter accepting string slices
            pub fn location_str<S: Into<String>>(mut self, location: S) -> Self {
                self.location = Some(location.into());
                self
            }
            
            /// Set multiple string fields at once
            pub fn with_profile<S1, S2, S3>(mut self, username: S1, email: S2, full_name: S3) -> Self 
            where
                S1: Into<String>,
                S2: Into<String>, 
                S3: Into<String>,
            {
                self.username = Some(username.into());
                self.email = Some(email.into());
                self.full_name = Some(full_name.into());
                self
            }
            
            /// Conditional bio setting
            pub fn bio_if_some<S: Into<String>>(mut self, bio: Option<S>) -> Self {
                if let Some(b) = bio {
                    self.bio = Some(b.into());
                }
                self
            }
            
            /// Fluent verification setter
            pub fn verified(mut self) -> Self {
                self.is_verified = Some(true);
                self
            }
            
            /// Fluent unverified setter
            pub fn unverified(mut self) -> Self {
                self.is_verified = Some(false);
                self
            }
        }
    };
}

/// Result combinator extensions for builder patterns
pub trait BuilderResult<T> {
    /// Convert validation errors to app errors
    fn validation_error(self, message: &str) -> AppResult<T>;
    
    /// Provide default value on builder error
    fn or_default_build(self) -> AppResult<T> 
    where 
        T: Default;
}

impl<T> BuilderResult<T> for Result<T, String> {
    fn validation_error(self, message: &str) -> AppResult<T> {
        self.map_err(|e| crate::error::AppError::Validation(format!("{}: {}", message, e)))
    }
    
    fn or_default_build(self) -> AppResult<T> 
    where 
        T: Default 
    {
        match self {
            Ok(val) => Ok(val),
            Err(_) => Ok(T::default()),
        }
    }
}

/// Ergonomic patterns for common builder operations
pub struct BuilderPatterns;

impl BuilderPatterns {
    /// Create a builder with common defaults
    pub fn with_defaults<B: Default>(mut builder: B, setup: impl FnOnce(&mut B)) -> B {
        setup(&mut builder);
        builder
    }
    
    /// Batch create multiple entities with the same context
    pub async fn batch_create<B, E, C>(
        builders: Vec<B>, 
        context: C
    ) -> AppResult<Vec<E>>
    where
        B: ContextualBuilder<C, Output = E>,
        C: Clone,
    {
        let mut results = Vec::with_capacity(builders.len());
        for builder in builders {
            let entity = builder.create_with(context.clone()).await?;
            results.push(entity);
        }
        Ok(results)
    }
}

// Example usage patterns that would be generated for each entity:
/*
// Enhanced EntUser builder usage:
let user = EntUser::create(ctx)
    .with_profile("john_doe", "john@example.com", "John Doe")
    .bio_str("Software developer")
    .verified()
    .savex()
    .await?;

// Conditional field setting:
let user = EntUser::create(ctx)
    .username_str("jane_doe")
    .email_str("jane@example.com") 
    .bio_if_some(Some("Designer"))
    .chain(|b| if is_premium { b.verified() } else { b.unverified() })
    .savex()
    .await?;

// Batch creation:
let users = BuilderPatterns::batch_create(vec![
    EntUser::create().with_profile("user1", "u1@example.com", "User One"),
    EntUser::create().with_profile("user2", "u2@example.com", "User Two"),
], ctx).await?;
*/