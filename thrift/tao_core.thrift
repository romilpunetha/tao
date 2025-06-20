namespace rs tao_db.thrift

// TAO Core Data Structures - matching Meta's TAO architecture

// TAO Object - represents all entities in the system
struct TaoObject {
  1: required i64 id,
  2: required string object_type,
  3: required binary data,
  4: required i64 created,
  5: required i64 updated
}

// TAO Association - represents all relations/edges between entities
struct TaoAssociation {
  1: required i64 edge_id,
  2: required i64 source_id,
  3: required i64 target_id,
  4: required string association_type,
  5: optional binary association_data,
  6: required i64 created,
  7: required i64 updated,
  8: required i64 time_field  // TAO's special time attribute for creation-time locality
}

// TAO Index - for efficient queries like "all friends of user X"
struct TaoIndex {
  1: required i64 entity_id,
  2: required i64 edge_id,
  3: required i64 target_entity_id,
  4: required string association_type,
  5: required i64 created,
  6: required i64 updated
}

// TAO Query structures
struct TaoAssociationQuery {
  1: required i64 id1,
  2: optional i64 id2,
  3: required string assoc_type,
  4: optional i64 start_time,
  5: optional i64 end_time,
  6: optional i32 limit,
  7: optional i64 offset
}

struct TaoBatchObjectRequest {
  1: required list<i64> object_ids,
  2: required string object_type
}

struct TaoBatchAssociationRequest {
  1: required list<TaoAssociationQuery> queries
}