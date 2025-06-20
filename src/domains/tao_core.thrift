namespace rs tao_database.core

// Core TAO types and enums
typedef i64 TaoId
typedef i64 TaoTime
typedef string TaoType
typedef string AssocType

// TAO Association for edge relationships
struct TaoAssociation {
    1: required TaoId id1,          // Source entity ID
    2: required AssocType atype,    // Association type (e.g., "friendship", "like")
    3: required TaoId id2,          // Target entity ID
    4: required TaoTime time,       // Association timestamp
    5: optional string data,        // Optional metadata
}

// TAO Object for entities
struct TaoObject {
    1: required TaoId id,           // Entity ID
    2: required TaoType otype,      // Object type (e.g., "user", "post")
    3: required TaoTime time,       // Creation/update timestamp
    4: optional string data,        // Serialized entity data
}

// Association query result
struct TaoAssocQueryResult {
    1: required list<TaoAssociation> associations,
    2: optional string next_cursor,  // For pagination
}

// Object query result
struct TaoObjectQueryResult {
    1: required list<TaoObject> objects,
    2: optional string next_cursor,  // For pagination
}
