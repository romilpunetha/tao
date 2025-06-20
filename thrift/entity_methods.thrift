namespace rs tao_db.thrift

// Meta's Entity Framework - Database method signatures for entities

// Entity database interface that all Ents must implement
struct EntityOp {
  1: required string entity_type,
  2: required i64 entity_id,
  3: optional binary data,
  4: optional string operation, // "get", "create", "update", "delete"
}

// Batch operations for entities
struct EntityBatchOp {
  1: required string entity_type,
  2: required list<i64> entity_ids,
  3: optional i32 limit,
  4: optional i64 offset,
}

// Entity query parameters
struct EntityQuery {
  1: required string entity_type,
  2: optional i32 limit,
  3: optional i64 offset,
  4: optional map<string, string> filters,
}