namespace rs ent_post

include "tao_core.thrift"

// Validated type for content
typedef string Content (
    min_length = "1",
    max_length = "10000",
)

// EntPost - Generated from Ent Schema with TAO functionality
struct EntPost {
  1: required i64 author_id,
  2: required Content content,
  3: optional string media_url,
  4: required i64 created_time,
  5: optional i64 updated_time,
  6: required string post_type,
  7: optional string visibility,
  8: required i32 like_count,
  9: required i32 comment_count,
  10: required i32 share_count,
  11: optional string tags,
  12: optional string mentions,
}

// TAO service for EntPost - handles persistence and relationship traversal
service EntPostService {
  // Validate entity according to schema constraints
  list<string> validate(1: EntPost entity),

  // Save entity to TAO database
  i64 save(1: EntPost entity),

  // Load entity from TAO database
  EntPost load(1: i64 entity_id),

  // Load multiple entities
  list<EntPost> load_multi(1: list<i64> entity_ids),

  // Get EntUser IDs via author edge
  i64 get_author_ids(1: i64 entity_id),

  // Get EntComment IDs via comments edge
  list<i64> get_comments_ids(1: i64 entity_id),

  // Get EntUser IDs via liked_by edge
  i64 get_liked_by_ids(1: i64 entity_id),

  // Get EntUser IDs via mentioned_users edge
  list<i64> get_mentioned_users_ids(1: i64 entity_id),

  // Get EntPage IDs via appears_on_pages edge
  list<i64> get_appears_on_pages_ids(1: i64 entity_id),

  // Get EntGroup IDs via shared_in_groups edge
  list<i64> get_shared_in_groups_ids(1: i64 entity_id),

  // Get EntEvent IDs via related_events edge
  list<i64> get_related_events_ids(1: i64 entity_id),

  // Load serialized EntUser data via author edge
  binary gen_author(1: i64 entity_id),

  // Load serialized EntComment data via comments edge
  list<binary> gen_comments(1: i64 entity_id),

  // Load serialized EntUser data via liked_by edge
  binary gen_liked_by(1: i64 entity_id),

  // Load serialized EntUser data via mentioned_users edge
  list<binary> gen_mentioned_users(1: i64 entity_id),

  // Load serialized EntPage data via appears_on_pages edge
  list<binary> gen_appears_on_pages(1: i64 entity_id),

  // Load serialized EntGroup data via shared_in_groups edge
  list<binary> gen_shared_in_groups(1: i64 entity_id),

  // Load serialized EntEvent data via related_events edge
  list<binary> gen_related_events(1: i64 entity_id),

}

