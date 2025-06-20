namespace rs tao_db.schemas.ent_page

// EntPage - Page entity data  
struct EntPage {
  1: required string name,
  2: optional string description,
  3: required string category,
  4: required i64 created_time,
  5: optional string website,
  6: required i32 follower_count,
  7: optional string profile_picture_url,
  8: required bool is_verified
}