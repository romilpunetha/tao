namespace rs tao_db.schemas.ent_comment

// EntComment - Comment entity data
struct EntComment {
  1: required i64 post_id,
  2: required i64 author_id,
  3: required string content,
  4: required i64 created_time,
  5: optional i64 updated_time,
  6: optional i64 parent_comment_id,
  7: required i32 like_count
}