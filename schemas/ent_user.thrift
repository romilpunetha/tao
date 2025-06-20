namespace rs tao_db.schemas.ent_user

// EntUser - User entity data
struct EntUser {
  1: required string username,
  2: required string email,
  3: optional string full_name,
  4: optional string bio,
  5: optional string profile_picture_url,
  6: required i64 created_time,
  7: optional i64 last_active_time,
  8: required bool is_verified,
  9: optional string location
}