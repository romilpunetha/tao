namespace rs tao_db.schemas.ent_event

// EntEvent - TAO Entity Schema
struct EntEvent {
  1: required i64 created_time, // Unix timestamp when entity was created
  2: optional i64 updated_time, // Unix timestamp when entity was last updated
  3: required string title
  4: optional string description
  5: required i64 start_time
}