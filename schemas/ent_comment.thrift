namespace rs tao_database.models.ent_comment

// EntComment - Generated from Ent Schema
struct EntComment {
  1: required i64 author_id,
  2: required i64 post_id,
  3: required string content,
  4: required i64 created_time,
}
