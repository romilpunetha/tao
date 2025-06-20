namespace rs ent_comment

include "tao_core.thrift"

// EntComment - Generated from Ent Schema with TAO functionality
struct EntComment {
  1: required i64 author_id,
  2: required i64 post_id,
  3: required string content,
  4: required i64 created_time,
}

// TAO service for EntComment - handles persistence and relationship traversal
service EntCommentService {
  // Validate entity according to schema constraints
  list<string> validate(1: EntComment entity),

  // Save entity to TAO database
  i64 save(1: EntComment entity),

  // Load entity from TAO database
  EntComment load(1: i64 entity_id),

  // Load multiple entities
  list<EntComment> load_multi(1: list<i64> entity_ids),

  // Get EntUser IDs via author edge
  i64 get_author_ids(1: i64 entity_id),

  // Get EntPost IDs via post edge
  i64 get_post_ids(1: i64 entity_id),

  // Load serialized EntUser data via author edge
  binary gen_author(1: i64 entity_id),

  // Load serialized EntPost data via post edge
  binary gen_post(1: i64 entity_id),

}

