// Meta's Entity (Ent) System - Pure Entity Architecture

// Re-export all generated types
pub mod associations;
pub mod tao_core;

// Individual entity modules
pub mod ent_user;
pub mod ent_event;
pub mod ent_post;
pub mod ent_comment;
pub mod ent_group;
pub mod ent_page;

// Re-export entity types and TAO core
pub use associations::{Friendship, Follow, Like, PostAuthor};
pub use ent_user::EntUser;
pub use ent_event::EntEvent;
pub use ent_post::EntPost;
pub use ent_comment::EntComment;
pub use ent_group::EntGroup;
pub use ent_page::EntPage;
pub use tao_core::{TaoObject, TaoAssociation, TaoIndex, TaoAssociationQuery};

// Meta's Entity Type system - everything is an entity
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EntityType {
    EntUser,
    EntPost,
    EntComment,
    EntGroup,
    EntPage,

    EntEvent,}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::EntUser => "ent_user",
            EntityType::EntPost => "ent_post", 
            EntityType::EntComment => "ent_comment",
            EntityType::EntGroup => "ent_group",
            EntityType::EntPage => "ent_page",
        
            EntityType::EntEvent => "ent_event",}
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Hash, Eq)]
pub enum AssociationType {
    Friendship,
    Follow,
    Like,
    PostAuthor,
    // Inverse association types for bidirectional relationships
    FollowedBy,
    LikedBy,
    // Additional association types for comprehensive entity relationships
    Membership,
    EventAttendance,
    CommentParent,
    Comments,
    MentionedUsers,
    AppearsOnPages,
    SharedInGroups,
    RelatedEvents,
    Attendees,
    RelatedPosts,
}

impl AssociationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssociationType::Friendship => "friendship",
            AssociationType::Follow => "follow",
            AssociationType::Like => "like", 
            AssociationType::PostAuthor => "post_author",
            AssociationType::FollowedBy => "followed_by",
            AssociationType::LikedBy => "liked_by",
            AssociationType::Membership => "membership",
            AssociationType::EventAttendance => "event_attendance",
            AssociationType::CommentParent => "comment_parent",
            AssociationType::Comments => "comments",
            AssociationType::MentionedUsers => "mentioned_users",
            AssociationType::AppearsOnPages => "appears_on_pages",
            AssociationType::SharedInGroups => "shared_in_groups",
            AssociationType::RelatedEvents => "related_events",
            AssociationType::Attendees => "attendees",
            AssociationType::RelatedPosts => "related_posts",
        }
    }
}

// Legacy compatibility for query structure
#[derive(Debug, Clone)]
pub struct AssociationQuery {
    pub id1: i64,
    pub id2: Option<i64>,
    pub assoc_type: String,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<i32>,
    pub offset: Option<i64>,
}