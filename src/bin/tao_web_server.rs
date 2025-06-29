// TAO Web Server - Complete REST API for TAO social graph database
// Provides endpoints for creating users, relationships, and visualizing the graph

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

use sqlx::postgres::PgPoolOptions;
use tao_database::domains::user::{builder::EntUserBuilder, entity::EntUser}; // Separate import
use tao_database::ent_framework::Entity;
use tao_database::{
    error::{AppError, AppResult},
    infrastructure::{
        association_registry::AssociationRegistry,
        database::{DatabaseInterface, PostgresDatabase},
        initialize_cache_default, initialize_metrics_default,
        query_router::{QueryRouterConfig, TaoQueryRouter},
        shard_topology::{ShardHealth, ShardInfo},
        tao::Tao,
        tao_core::{create_tao_association, current_time_millis, TaoId, TaoOperations},
        write_ahead_log::{TaoWriteAheadLog, WalConfig},
    },
};

// use tao_database::data_seeder;

// API request/response types
#[derive(Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    bio: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CreateRelationshipRequest {
    from_user_id: TaoId,
    to_user_id: TaoId,
    relationship_type: String, // "friendship", "follows", "blocks", etc.
}

#[derive(Serialize, Deserialize)]
struct UserResponse {
    id: TaoId,
    name: Option<String>,
    email: Option<String>,
    bio: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RelationshipResponse {
    id1: TaoId,
    id2: TaoId,
    relationship_type: String,
    created_at: i64,
}

#[derive(Serialize, Deserialize)]
struct GraphResponse {
    users: Vec<UserResponse>,
    relationships: Vec<RelationshipResponse>,
}

#[derive(Serialize, Deserialize)]
struct GraphDataResponse {
    objects: Vec<TaoObjectJson>,
    associations: Vec<TaoAssociationJson>,
}

#[derive(Serialize, Deserialize)]
struct TaoObjectJson {
    id: TaoId,
    otype: String,
    data: serde_json::Value,
    created_time: i64,
    updated_time: i64,
    version: u64,
}

#[derive(Serialize, Deserialize)]
struct TaoAssociationJson {
    id1: TaoId,
    atype: String,
    id2: TaoId,
    time: i64,
    data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct GraphQueryParams {
    limit: Option<u32>,
}

// Application state
#[derive(Clone)]
struct AppState {
    tao: Arc<Tao>,
}

// API Handlers
async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    info!("Creating user: {}", request.name);

    let user_builder = EntUser::create()
        .username(request.name.to_lowercase().replace(" ", "_"))
        .email(request.email.clone())
        .full_name(request.name.clone())
        .bio(request.bio.unwrap_or("".to_string()))
        .is_verified(true);

    match user_builder.save(&state.tao).await {
        Ok(user) => {
            info!(
                "Created user: {} (ID: {})",
                user.full_name.as_deref().unwrap_or("Unknown"), // Handle Option<String> for logging
                user.id
            );
            let response = ApiResponse {
                success: true,
                data: Some(UserResponse {
                    id: user.id,
                    name: user.full_name,
                    email: Some(user.email),
                    bio: user.bio,
                }),
                error: None,
            };
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            warn!("Failed to create user: {}", e);
            let response = ApiResponse::<UserResponse> {
                success: false,
                data: None,
                error: Some(format!("Failed to create user: {}", e)),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn create_relationship(
    State(state): State<AppState>,
    Json(request): Json<CreateRelationshipRequest>,
) -> impl IntoResponse {
    info!(
        "Creating relationship: {} -> {} ({})",
        request.from_user_id, request.to_user_id, request.relationship_type
    );

    let association = create_tao_association(
        request.from_user_id,
        request.relationship_type.clone(),
        request.to_user_id,
        None,
    );

    match state.tao.assoc_add(association.clone()).await {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                data: Some(RelationshipResponse {
                    id1: request.from_user_id,
                    id2: request.to_user_id,
                    relationship_type: request.relationship_type,
                    created_at: association.time,
                }),
                error: None,
            };
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            warn!("Failed to create relationship: {}", e);
            let response = ApiResponse::<RelationshipResponse> {
                success: false,
                data: None,
                error: Some(format!("Failed to create relationship: {}", e)),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn get_user(State(state): State<AppState>, Path(user_id): Path<TaoId>) -> impl IntoResponse {
    let tao_ops_arc: Arc<dyn TaoOperations> = state.tao.clone();
    match EntUser::gen_nullable(&tao_ops_arc, Some(user_id)).await {
        Ok(Some(user)) => {
            let response = ApiResponse {
                success: true,
                data: Some(UserResponse {
                    id: user.id,
                    name: user.full_name,
                    email: Some(user.email),
                    bio: user.bio,
                }),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            let response = ApiResponse::<UserResponse> {
                success: false,
                data: None,
                error: Some("User not found".to_string()),
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(e) => {
            warn!("Failed to get user {}: {}", user_id, e);
            let response = ApiResponse::<UserResponse> {
                success: false,
                data: None,
                error: Some(format!("Failed to get user: {}", e)),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn get_all_users(
    State(state): State<AppState>,
    Query(params): Query<GraphQueryParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100);

    match state
        .tao
        .get_all_objects_of_type("user".to_string(), Some(limit))
        .await
    {
        Ok(user_objs) => {
            let mut users = Vec::new();
            for user_obj in user_objs {
                if let Ok(user_data) = serde_json::from_slice::<serde_json::Value>(&user_obj.data) {
                    let id = user_obj.id; // Added this line
                    users.push(UserResponse {
                        id,
                        name: user_data["name"].as_str().map(|s| s.to_string()),
                        email: user_data["email"].as_str().map(|s| s.to_string()),
                        bio: user_data["bio"].as_str().map(|s| s.to_string()),
                    });
                }
            }

            let response = ApiResponse {
                success: true,
                data: Some(users),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            warn!("Failed to get all users: {}", e);
            let response = ApiResponse::<Vec<UserResponse>> {
                success: false,
                data: None,
                error: Some(format!("Failed to get users: {}", e)),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn get_graph(
    State(state): State<AppState>,
    Query(params): Query<GraphQueryParams>,
) -> (StatusCode, Json<ApiResponse<GraphResponse>>) {
    let limit = params.limit.unwrap_or(100);

    info!("Fetching graph data with limit: {}", limit);

    let mut users = Vec::new();
    let mut relationships = Vec::new();

    // Fetch all users
    let all_users = match state
        .tao
        .get_all_objects_of_type("user".to_string(), Some(limit))
        .await
    {
        Ok(users) => users,
        Err(e) => {
            let response = ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to fetch users: {}", e)),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    // Process users and fetch their relationships
    for user_obj in all_users {
        let id = user_obj.id;
        if let Ok(user_data) = serde_json::from_slice::<serde_json::Value>(&user_obj.data) {
            users.push(UserResponse {
                id,
                name: user_data["name"].as_str().map(|s| s.to_string()),
                email: user_data["email"].as_str().map(|s| s.to_string()),
                bio: user_data["bio"].as_str().map(|s| s.to_string()),
            });

            // Fetch relationships for this user
            if let Ok(assocs) = state
                .tao
                .get_neighbor_ids(id, "friendship".to_string(), Some(limit))
                .await
            {
                for neighbor_id in assocs {
                    relationships.push(RelationshipResponse {
                        id1: id,
                        id2: neighbor_id,
                        relationship_type: "friendship".to_string(),
                        created_at: current_time_millis(),
                    });
                }
            }
        }
    }

    let response = ApiResponse {
        success: true,
        data: Some(GraphResponse {
            users,
            relationships,
        }),
        error: None,
    };

    (StatusCode::OK, Json(response))
}

async fn get_graph_data(State(state): State<AppState>) -> impl IntoResponse {
    info!("Fetching complete graph data from all shards");

    match state.tao.get_graph_data().await {
        Ok((objects, associations)) => {
            let mut json_objects = Vec::new();
            let mut json_associations = Vec::new();

            // Convert objects to JSON format
            for obj in objects {
                let data_json = match serde_json::from_slice::<serde_json::Value>(&obj.data) {
                    Ok(json) => json,
                    Err(_) => serde_json::json!({ "raw": String::from_utf8_lossy(&obj.data) }),
                };

                json_objects.push(TaoObjectJson {
                    id: obj.id,
                    otype: obj.otype,
                    data: data_json,
                    created_time: obj.created_time,
                    updated_time: obj.updated_time,
                    version: obj.version,
                });
            }

            // Convert associations to JSON format
            for assoc in associations {
                let data_json = if let Some(ref data) = assoc.data {
                    match serde_json::from_slice::<serde_json::Value>(&data) {
                        Ok(json) => Some(json),
                        Err(_) => {
                            Some(serde_json::json!({ "raw": String::from_utf8_lossy(&data) }))
                        }
                    }
                } else {
                    None
                };

                json_associations.push(TaoAssociationJson {
                    id1: assoc.id1,
                    atype: assoc.atype,
                    id2: assoc.id2,
                    time: assoc.time,
                    data: data_json,
                });
            }

            let response = ApiResponse {
                success: true,
                data: Some(GraphDataResponse {
                    objects: json_objects,
                    associations: json_associations,
                }),
                error: None,
            };

            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            warn!("Failed to fetch graph data: {}", e);
            let response = ApiResponse::<GraphDataResponse> {
                success: false,
                data: None,
                error: Some(format!("Failed to fetch graph data: {}", e)),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn serve_frontend() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "TAO Graph Database",
        "timestamp": current_time_millis()
    }))
}

async fn seed_data_handler(State(state): State<AppState>) -> impl IntoResponse {
    info!("Seeding sample data...");

    // Create sample users using EntUserBuilder
    let sample_users_data = vec![
        (
            "Grace Hopper",
            "grace@example.com",
            Some("Full-stack developer and open source contributor"),
            true,
        ),
        (
            "Alice Johnson",
            "alice@example.com",
            Some("Software engineer who loves hiking and photography"),
            true,
        ),
        (
            "Bob Smith",
            "bob@example.com",
            Some("Product manager with a passion for cycling"),
            true,
        ),
        (
            "Charlie Brown",
            "charlie@example.com",
            Some("Designer focused on user experience"),
            false,
        ),
        (
            "Diana Prince",
            "diana@example.com",
            Some("Data scientist exploring machine learning"),
            true,
        ),
        (
            "Eve Wilson",
            "eve@example.com",
            Some("Marketing specialist who enjoys cooking"),
            false,
        ),
        (
            "Frank Castle",
            "frank@example.com",
            Some("DevOps engineer with security expertise"),
            true,
        ),
        (
            "Henry Ford",
            "henry@example.com",
            Some("Engineering manager leading innovative projects"),
            false,
        ),
    ];

    let mut users: Vec<EntUser> = Vec::new();

    for (name, email, bio, is_verified) in sample_users_data {
        let user_builder = EntUser::create()
            .username(name.to_lowercase().replace(" ", "_"))
            .email(email.to_string())
            .full_name(name.to_string())
            .bio(bio.unwrap_or("").to_string())
            .is_verified(is_verified);

        match user_builder.save(&state.tao).await {
            Ok(user) => {
                info!("Created EntUser: {} (ID: {})", user.username, user.id);
                users.push(user);
            }
            Err(e) => {
                warn!("Failed to create EntUser {}: {}", name, e);
                let response = ApiResponse::<String> {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to create user {}: {}", name, e)),
                };
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
            }
        }
    }

    // Create sample relationships using Ent-specific methods
    if users.len() >= 2 {
        let relationships = vec![
            (0, 1, "friendship"),
            (0, 2, "follows"),
            (1, 2, "friendship"),
            (2, 3, "follows"),
            (3, 4, "friendship"),
            (4, 5, "follows"),
            (1, 5, "friendship"),
            (0, 6, "follows"),
            (6, 7, "friendship"),
            (3, 7, "follows"),
            (5, 7, "friendship"),
            (2, 6, "follows"),
        ];

        for (from_idx, to_idx, rel_type) in relationships {
            if from_idx < users.len() && to_idx < users.len() {
                let from_user = &users[from_idx];
                let to_user_id = users[to_idx].id;

                let result = match rel_type {
                    "friendship" => from_user.add_friend(&state.tao, to_user_id).await,
                    "follows" => from_user.add_following(&state.tao, to_user_id).await,
                    _ => Ok(()), // Should not happen with defined types
                };

                match result {
                    Ok(_) => {
                        info!(
                            "Created {} relationship between {} and {}",
                            rel_type, from_user.id, to_user_id
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to create {} relationship between {} and {}: {}",
                            rel_type, from_user.id, to_user_id, e
                        );
                    }
                }
            }
        }
    }

    let response = ApiResponse {
        success: true,
        data: Some(format!(
            "Successfully seeded {} users with relationships",
            users.len()
        )),
        error: None,
    };
    (StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() -> AppResult<()> {
    info!("🚀 Starting TAO Web Server...");

    // Initialize databases for sharding
    let shard_urls = vec![
        "postgresql://postgres:password@localhost:5432/tao_shard_1".to_string(),
        "postgresql://postgres:password@localhost:5433/tao_shard_2".to_string(),
        "postgresql://postgres:password@localhost:5434/tao_shard_3".to_string(),
    ];

    let query_router = Arc::new(TaoQueryRouter::new(QueryRouterConfig::default()).await);

    for (i, url) in shard_urls.iter().enumerate() {
        info!("Initializing shard {} at {}", i + 1, url);
        let pool = PgPoolOptions::new()
            .max_connections(10) // Example value, adjust as needed
            .connect(url)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!(
                    "Failed to connect to database for shard {}: {}",
                    i + 1,
                    e
                ))
            })?;
        let database = PostgresDatabase::new(pool);
        database.initialize().await?; // Initialize tables for this specific shard
        let db_interface: Arc<dyn DatabaseInterface> = Arc::new(database);

        let shard_info = ShardInfo {
            shard_id: i as u16,
            connection_string: url.clone(),
            region: "local".to_string(),
            health: ShardHealth::Healthy,
            replicas: vec![],
            last_health_check: current_time_millis(),
            load_factor: 0.0,
        };
        query_router.add_shard(shard_info, db_interface).await?;
        info!("✅ Shard {} configured", i + 1);
    }
    info!("✅ All shards configured");

    // Create TAO with WAL
    let association_registry = Arc::new(AssociationRegistry::new());

    // Setup WAL
    let wal_config = WalConfig::default();
    let wal = Arc::new(TaoWriteAheadLog::new(wal_config, "/tmp/tao_web_wal").await?);

    // Initialize cache and metrics
    let cache = initialize_cache_default().await?;
    let metrics = initialize_metrics_default().await?;

    // Create TaoCore instance
    let tao_core = Arc::new(tao_database::infrastructure::tao_core::TaoCore::new(
        query_router.clone(),
        association_registry.clone(),
    ));

    // Initialize TAO with all components
    let tao = Arc::new(Tao::new(tao_core, wal, cache, metrics, false, false));
    info!("✅ TAO initialized with production features");

    // Application state
    let app_state = AppState { tao };

    // Build router with CORS for frontend
    let app = Router::new()
        .route("/", get(serve_frontend))
        .route("/api/health", get(health_check))
        .route("/api/users", get(get_all_users).post(create_user))
        .route("/api/users/{id}", get(get_user))
        .route("/api/relationships", post(create_relationship))
        .route("/api/graph", get(get_graph))
        .route("/api/graph-data", get(get_graph_data))
        .route("/api/seed", post(seed_data_handler))
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            ),
        )
        .with_state(app_state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("🌐 Server starting on http://{}", addr);
    info!("📊 Graph visualization available at http://{}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
