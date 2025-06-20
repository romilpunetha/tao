namespace rs tao_database.models.ent_user

// EntUser - Generated from Ent Schema
struct EntUser {
  1: required string username,
  2: required string email,
  3: required i64 created_time,
  4: optional string full_name,
  5: optional string bio,
  6: optional string profile_picture_url,
  7: optional i64 last_active_time,
  8: required bool is_verified,
  9: optional string location,
  10: optional string privacy_settings,
}
