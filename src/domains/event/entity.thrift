namespace rs tao_database.domains.event

// Include core TAO types
include "../tao_core.thrift"

// EntEvent - Enhanced entity with TAO functionality
// Field validation typedefs

struct EntEvent {
    1: required string name,
    2: optional string description,
    3: required i64 event_time,
    4: required i64 created_time,
}

// TAO service for EntEvent persistence and relationships
service EntEventService {
    // Core entity operations
EntEvent get(1: i64 entity_id),
list<EntEvent> get_many(1: list<i64> entity_ids),
EntEvent create(1: EntEvent entity),
void update(1: EntEvent entity),
bool delete(1: i64 entity_id),
bool exists(1: i64 entity_id),

    // Edge traversal methods
    EntUser get_attendees(1: i64 source_id),
i64 get_attendees_ids(1: i64 source_id),
    EntPost get_related_posts(1: i64 source_id),
i64 get_related_posts_ids(1: i64 source_id),
}

