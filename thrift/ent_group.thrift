namespace rs ent_group

include "tao_core.thrift"

// EntGroup - Generated from Ent Schema with TAO functionality
struct EntGroup {
  1: required string name,
  2: optional string description,
  3: required i64 created_time,
}

// TAO service for EntGroup - handles persistence and relationship traversal
service EntGroupService {
  // Validate entity according to schema constraints
  list<string> validate(1: EntGroup entity),

  // Save entity to TAO database
  i64 save(1: EntGroup entity),

  // Load entity from TAO database
  EntGroup load(1: i64 entity_id),

  // Load multiple entities
  list<EntGroup> load_multi(1: list<i64> entity_ids),

  // Get EntUser IDs via members edge
  i64 get_members_ids(1: i64 entity_id),

  // Get EntPost IDs via posts edge
  i64 get_posts_ids(1: i64 entity_id),

  // Load serialized EntUser data via members edge
  binary gen_members(1: i64 entity_id),

  // Load serialized EntPost data via posts edge
  binary gen_posts(1: i64 entity_id),

}

