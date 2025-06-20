namespace rs tao_database.domains.post

// Include core TAO types
include "../tao_core.thrift"

// EntPost - Enhanced entity with TAO functionality
// Field validation typedefs
typedef string EntPost_CONTENT

struct EntPost {
    1: required i64 author_id,
    2: required string content,
    3: optional string media_url,
    4: required i64 created_time,
    5: optional i64 updated_time,
    6: required string post_type,
    7: optional string visibility,
    8: required i32 like_count,
    9: required i32 comment_count,
    10: required i32 share_count,
    11: optional string tags,
    12: optional string mentions,
}

// TAO service for EntPost persistence and relationships
service EntPostService {
    // Core entity operations
EntPost get(1: i64 entity_id),
list<EntPost> get_many(1: list<i64> entity_ids),
EntPost create(1: EntPost entity),
void update(1: EntPost entity),
bool delete(1: i64 entity_id),
bool exists(1: i64 entity_id),

    // Edge traversal methods
    EntUser get_author(1: i64 source_id),
i64 get_author_ids(1: i64 source_id),
    list<EntComment> get_comments(1: i64 source_id),
list<i64> get_comments_ids(1: i64 source_id),
    EntUser get_liked_by(1: i64 source_id),
i64 get_liked_by_ids(1: i64 source_id),
    list<EntUser> get_mentioned_users(1: i64 source_id),
list<i64> get_mentioned_users_ids(1: i64 source_id),
    list<EntPage> get_appears_on_pages(1: i64 source_id),
list<i64> get_appears_on_pages_ids(1: i64 source_id),
    list<EntGroup> get_shared_in_groups(1: i64 source_id),
list<i64> get_shared_in_groups_ids(1: i64 source_id),
    list<EntEvent> get_related_events(1: i64 source_id),
list<i64> get_related_events_ids(1: i64 source_id),
}

