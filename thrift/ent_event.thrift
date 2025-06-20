namespace rs ent_event

include "tao_core.thrift"

// EntEvent - Generated from Ent Schema with TAO functionality
struct EntEvent {
  1: required string name,
  2: optional string description,
  3: required i64 event_time,
  4: required i64 created_time,
}

// TAO service for EntEvent - handles persistence and relationship traversal
service EntEventService {
  // Validate entity according to schema constraints
  list<string> validate(1: EntEvent entity),

  // Save entity to TAO database
  i64 save(1: EntEvent entity),

  // Load entity from TAO database
  EntEvent load(1: i64 entity_id),

  // Load multiple entities
  list<EntEvent> load_multi(1: list<i64> entity_ids),

  // Get EntUser IDs via attendees edge
  i64 get_attendees_ids(1: i64 entity_id),

  // Get EntPost IDs via related_posts edge
  i64 get_related_posts_ids(1: i64 entity_id),

  // Load serialized EntUser data via attendees edge
  binary gen_attendees(1: i64 entity_id),

  // Load serialized EntPost data via related_posts edge
  binary gen_related_posts(1: i64 entity_id),

}

