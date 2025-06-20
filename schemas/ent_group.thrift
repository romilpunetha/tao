namespace rs tao_db.schemas.ent_group

// EntGroup - Group entity data
struct EntGroup {
  1: required string name,
  2: optional string description,
  3: required i64 created_time,
  4: required i64 creator_id,
  5: required string privacy, // "public", "closed", "secret"
  6: required i32 member_count,
  7: optional string cover_photo_url
}