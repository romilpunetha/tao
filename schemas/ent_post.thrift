namespace rs tao_database.models.ent_post

// EntPost - Generated from Ent Schema
struct EntPost {
  1: required i64 author_id,
  2: required string content,
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
