namespace rs ent_page

include "tao_core.thrift"

// EntPage - Generated from Ent Schema with TAO functionality
struct EntPage {
  1: required string name,
  2: optional string description,
  3: required i64 created_time,
}

// TAO service for EntPage - handles persistence and relationship traversal
service EntPageService {
  // Validate entity according to schema constraints
  list<string> validate(1: EntPage entity),

  // Save entity to TAO database
  i64 save(1: EntPage entity),

  // Load entity from TAO database
  EntPage load(1: i64 entity_id),

  // Load multiple entities
  list<EntPage> load_multi(1: list<i64> entity_ids),

  // Get EntUser IDs via followers edge
  i64 get_followers_ids(1: i64 entity_id),

  // Get EntPost IDs via posts edge
  i64 get_posts_ids(1: i64 entity_id),

  // Load serialized EntUser data via followers edge
  binary gen_followers(1: i64 entity_id),

  // Load serialized EntPost data via posts edge
  binary gen_posts(1: i64 entity_id),

}

