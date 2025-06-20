namespace rs tao_core

// Core TAO object type
struct TaoObject {
  1: required i64 id,
  2: required string object_type,
  3: required binary data,
  4: required i64 created_time,
  5: required i64 updated_time,
}

// TAO association type
struct TaoAssociation {
  1: required i64 id,
  2: required i64 id1,
  3: required i64 id2,
  4: required string assoc_type,
  5: optional binary data,
  6: required i64 created_time,
  7: required i64 updated_time,
  8: optional i64 time1,
  9: optional i64 time2,
}

// TAO association query
struct TaoAssociationQuery {
  1: required i64 id1,
  2: required string assoc_type,
  3: optional list<i64> id2s,
  4: optional i64 time_low,
  5: optional i64 time_high,
  6: optional i32 limit,
}

// TAO index for performance
struct TaoIndex {
  1: required string name,
  2: required list<string> fields,
  3: required bool unique,
}
