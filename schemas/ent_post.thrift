namespace rs tao_db.schemas.ent_post

// EntPost - Post entity data  
struct EntPost {
  1: required i64 author_id,
  2: required string content,
  3: optional string media_url,
  4: required i64 created_time,
  5: optional i64 updated_time,
  6: required string post_type, // "text", "photo", "video", "link"
  7: optional string visibility, // "public", "friends", "private"
  8: required i32 like_count,
  9: required i32 comment_count,
  10: required i32 share_count
}