namespace rs tao_database.domains.group

// Include core TAO types
include "../tao_core.thrift"

// EntGroup - Enhanced entity with TAO functionality
// Field validation typedefs

struct EntGroup {
    1: required string name,
    2: optional string description,
    3: required i64 created_time,
}

// TAO service for EntGroup persistence and relationships
service EntGroupService {
    // Core entity operations
EntGroup get(1: i64 entity_id),
list<EntGroup> get_many(1: list<i64> entity_ids),
EntGroup create(1: EntGroup entity),
void update(1: EntGroup entity),
bool delete(1: i64 entity_id),
bool exists(1: i64 entity_id),

    // Edge traversal methods
    EntUser get_members(1: i64 source_id),
i64 get_members_ids(1: i64 source_id),
    EntPost get_posts(1: i64 source_id),
i64 get_posts_ids(1: i64 source_id),
}

