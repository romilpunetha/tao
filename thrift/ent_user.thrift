namespace rs ent_user

include "tao_core.thrift"

// Validated type for username
typedef string Username (
    min_length = "3",
    max_length = "30",
    pattern = "^[a-zA-Z0-9_]+$",
)

// Validated type for email
typedef string Email (
    pattern = "^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$",
)

// Validated type for full_name
typedef string FullName (
    max_length = "100",
)

// Validated type for bio
typedef string Bio (
    max_length = "500",
)

// EntUser - Generated from Ent Schema with TAO functionality
struct EntUser {
  1: required Username username,
  2: required Email email,
  3: required i64 created_time,
  4: optional FullName full_name,
  5: optional Bio bio,
  6: optional string profile_picture_url,
  7: optional i64 last_active_time,
  8: required bool is_verified,
  9: optional string location,
  10: optional string privacy_settings,
}

// TAO service for EntUser - handles persistence and relationship traversal
service EntUserService {
  // Validate entity according to schema constraints
  list<string> validate(1: EntUser entity),

  // Save entity to TAO database
  i64 save(1: EntUser entity),

  // Load entity from TAO database
  EntUser load(1: i64 entity_id),

  // Load multiple entities
  list<EntUser> load_multi(1: list<i64> entity_ids),

  // Get EntUser IDs via friends edge
  list<i64> get_friends_ids(1: i64 entity_id),

  // Get EntUser IDs via following edge
  list<i64> get_following_ids(1: i64 entity_id),

  // Get EntUser IDs via followers edge
  i64 get_followers_ids(1: i64 entity_id),

  // Get EntPost IDs via posts edge
  list<i64> get_posts_ids(1: i64 entity_id),

  // Get EntPost IDs via liked_posts edge
  list<i64> get_liked_posts_ids(1: i64 entity_id),

  // Get EntGroup IDs via groups edge
  list<i64> get_groups_ids(1: i64 entity_id),

  // Get EntPage IDs via followed_pages edge
  list<i64> get_followed_pages_ids(1: i64 entity_id),

  // Get EntEvent IDs via attending_events edge
  list<i64> get_attending_events_ids(1: i64 entity_id),

  // Load serialized EntUser data via friends edge
  list<binary> gen_friends(1: i64 entity_id),

  // Load serialized EntUser data via following edge
  list<binary> gen_following(1: i64 entity_id),

  // Load serialized EntUser data via followers edge
  binary gen_followers(1: i64 entity_id),

  // Load serialized EntPost data via posts edge
  list<binary> gen_posts(1: i64 entity_id),

  // Load serialized EntPost data via liked_posts edge
  list<binary> gen_liked_posts(1: i64 entity_id),

  // Load serialized EntGroup data via groups edge
  list<binary> gen_groups(1: i64 entity_id),

  // Load serialized EntPage data via followed_pages edge
  list<binary> gen_followed_pages(1: i64 entity_id),

  // Load serialized EntEvent data via attending_events edge
  list<binary> gen_attending_events(1: i64 entity_id),

}

