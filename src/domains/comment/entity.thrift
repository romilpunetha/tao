namespace rs tao_database.domains.comment

// Include core TAO types
include "../tao_core.thrift"

// EntComment - Enhanced entity with TAO functionality
// Field validation typedefs

struct EntComment {
    1: required i64 author_id,
    2: required i64 post_id,
    3: required string content,
    4: required i64 created_time,
}

// TAO service for EntComment persistence and relationships
service EntCommentService {
    // Core entity operations
EntComment get(1: i64 entity_id),
list<EntComment> get_many(1: list<i64> entity_ids),
EntComment create(1: EntComment entity),
void update(1: EntComment entity),
bool delete(1: i64 entity_id),
bool exists(1: i64 entity_id),

    // Edge traversal methods
    EntUser get_author(1: i64 source_id),
i64 get_author_ids(1: i64 source_id),
    EntPost get_post(1: i64 source_id),
i64 get_post_ids(1: i64 source_id),
}

