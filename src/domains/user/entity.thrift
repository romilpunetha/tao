namespace rs tao_database.domains.user

// Include core TAO types
include "../tao_core.thrift"

// EntUser - Enhanced entity with TAO functionality
// Field validation typedefs
typedef string EntUser_USERNAME
typedef string EntUser_EMAIL
typedef string EntUser_FULL_NAME
typedef string EntUser_BIO

struct EntUser {
    1: required string username,
    2: required string email,
    3: required i64 created_time,
    4: optional string full_name,
    5: optional string bio,
    6: optional string profile_picture_url,
    7: optional i64 last_active_time,
    8: required bool is_verified,
    9: optional string location,
    10: optional string privacy_settings,
}

// TAO service for EntUser persistence and relationships
service EntUserService {
    // Core entity operations
EntUser get(1: i64 entity_id),
list<EntUser> get_many(1: list<i64> entity_ids),
EntUser create(1: EntUser entity),
void update(1: EntUser entity),
bool delete(1: i64 entity_id),
bool exists(1: i64 entity_id),

    // Edge traversal methods
    list<EntUser> get_friends(1: i64 source_id),
list<i64> get_friends_ids(1: i64 source_id),
    list<EntUser> get_following(1: i64 source_id),
list<i64> get_following_ids(1: i64 source_id),
    EntUser get_followers(1: i64 source_id),
i64 get_followers_ids(1: i64 source_id),
    list<EntPost> get_posts(1: i64 source_id),
list<i64> get_posts_ids(1: i64 source_id),
    list<EntPost> get_liked_posts(1: i64 source_id),
list<i64> get_liked_posts_ids(1: i64 source_id),
    list<EntGroup> get_groups(1: i64 source_id),
list<i64> get_groups_ids(1: i64 source_id),
    list<EntPage> get_followed_pages(1: i64 source_id),
list<i64> get_followed_pages_ids(1: i64 source_id),
    list<EntEvent> get_attending_events(1: i64 source_id),
list<i64> get_attending_events_ids(1: i64 source_id),
}

