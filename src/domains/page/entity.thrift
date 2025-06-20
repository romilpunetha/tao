namespace rs tao_database.domains.page

// Include core TAO types
include "../tao_core.thrift"

// EntPage - Enhanced entity with TAO functionality
// Field validation typedefs

struct EntPage {
    1: required string name,
    2: optional string description,
    3: required i64 created_time,
}

// TAO service for EntPage persistence and relationships
service EntPageService {
    // Core entity operations
EntPage get(1: i64 entity_id),
list<EntPage> get_many(1: list<i64> entity_ids),
EntPage create(1: EntPage entity),
void update(1: EntPage entity),
bool delete(1: i64 entity_id),
bool exists(1: i64 entity_id),

    // Edge traversal methods
    EntUser get_followers(1: i64 source_id),
i64 get_followers_ids(1: i64 source_id),
    EntPost get_posts(1: i64 source_id),
i64 get_posts_ids(1: i64 source_id),
}

