// TAO Web Server - Complete REST API for TAO social graph database
// Provides endpoints for creating users, relationships, and visualizing the graph

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
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
use tao_database::domains::user::EntUser;
use tao_database::framework::entity::ent_trait::Entity;
use tao_database::{
    error::{AppError, AppResult},
    infrastructure::{
        association_registry::AssociationRegistry,
        database::database::{DatabaseInterface, PostgresDatabase},
        global_tao::{get_global_tao, set_global_tao},
        query_router::{QueryRouterConfig, TaoQueryRouter},
        shard_topology::{ShardHealth, ShardInfo},
        tao_core::tao::Tao,
        tao_core::tao_core::{create_tao_association, current_time_millis, TaoId, TaoOperations},
    },
};

// Import new graph models
use tao_database::models::graph_models::{GraphData, GraphEdge, GraphNode};

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
    username: String,
    email: String,
    full_name: Option<String>,
    bio: Option<String>,
    is_verified: bool,
    location: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RelationshipResponse {
    id1: TaoId,
    id2: TaoId,
    relationship_type: String,
    created_at: i64,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// Application state (empty as Tao is global)
#[derive(Clone)]
struct AppState {}

// API Handlers
async fn create_user(
    State(_state): State<AppState>, // _state to indicate it's unused
    Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    info!("Creating user: {}", request.name);

    let user_builder = EntUser::create()
        .username(request.name.to_lowercase().replace(" ", "_"))
        .email(request.email.clone())
        .full_name(request.name.clone())
        .bio(request.bio.unwrap_or("".to_string()))
        .is_verified(true);

    match user_builder.savex().await {
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
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    bio: user.bio,
                    is_verified: user.is_verified,
                    location: user.location,
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

async fn create_relationship(Json(request): Json<CreateRelationshipRequest>) -> impl IntoResponse {
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

    let tao = match get_global_tao() {
        Ok(t) => t.clone(),
        Err(e) => {
            warn!("Failed to get global TAO instance: {}", e);
            let response = ApiResponse::<RelationshipResponse> {
                success: false,
                data: None,
                error: Some(format!("Failed to get TAO: {}", e)),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };
    match tao.assoc_add(association.clone()).await {
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

async fn get_user(Path(user_id): Path<TaoId>) -> impl IntoResponse {
    // Tao operations are now accessed via get_global_tao()
    let tao = get_global_tao().expect("TAO not initialized");
    let tao_ops: Arc<dyn TaoOperations> = tao.clone();
    match EntUser::gen_nullable(&tao_ops, Some(user_id)).await {
        Ok(Some(user)) => {
            let response = ApiResponse {
                success: true,
                data: Some(UserResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    bio: user.bio,
                    is_verified: user.is_verified,
                    location: user.location,
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

async fn get_all_users() -> impl IntoResponse {
    let tao = get_global_tao().expect("TAO not initialized");
    let tao_ops: Arc<dyn TaoOperations> = tao.clone();
    match EntUser::gen_all(&tao_ops).await {
        Ok(user_objs) => {
            let mut users = Vec::new();
            for user in user_objs {
                users.push(UserResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    bio: user.bio,
                    is_verified: user.is_verified,
                    location: user.location,
                });
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

async fn get_graph_data() -> impl IntoResponse {
    info!("Fetching graph data.");

    let tao = get_global_tao().expect("TAO not initialized");
    let tao_ops: Arc<dyn TaoOperations> = tao.clone();
    let users = match EntUser::gen_all(&tao_ops).await {
        Ok(users) => users,
        Err(e) => {
            warn!("Failed to get all users for graph data: {}", e);
            let response = ApiResponse::<GraphData> {
                success: false,
                data: None,
                error: Some(format!("Failed to get graph data: {}", e)),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    let mut graph_nodes = Vec::with_capacity(users.len());
    let mut graph_edges = Vec::new();
    let mut relationship_futures = Vec::new();

    // Create nodes and collect relationship futures
    for user in &users {
        graph_nodes.push(GraphNode {
            id: user.id.to_string(),
            name: user
                .full_name
                .clone()
                .unwrap_or_else(|| user.username.clone()),
            node_type: EntUser::ENTITY_TYPE.to_string(),
            verified: user.is_verified,
        });

        println!(
            "User id : {}\nfollowers : {:?}\nfollowing : {:?}",
            user.id,
            user.get_friends().await,
            user.get_following().await
        );

        // Collect futures for batch processing
        relationship_futures.push(async move {
            let user_id_str = user.id.to_string();
            let mut edges = Vec::new();

            // Get friends with error logging
            match user.get_friends().await {
                Ok(friends) => {
                    for friend in friends {
                        edges.push(GraphEdge {
                            source: user_id_str.clone(),
                            target: friend.id.to_string(),
                            edge_type: "friendship".to_string(),
                            weight: 1.0,
                        });
                    }
                }
                Err(e) => warn!("Failed to get friends for user {}: {}", user.id, e),
            }

            // Get following with error logging
            match user.get_following().await {
                Ok(following) => {
                    for followed in following {
                        edges.push(GraphEdge {
                            source: user_id_str.clone(),
                            target: followed.id.to_string(),
                            edge_type: "follows".to_string(),
                            weight: 0.5,
                        });
                    }
                }
                Err(e) => warn!("Failed to get following for user {}: {}", user.id, e),
            }

            edges
        });
    }

    // Execute all relationship queries concurrently
    let edge_results = futures::future::join_all(relationship_futures).await;

    println!("Edges : {:?}", edge_results);

    // Flatten results into single edge vector
    for edges in edge_results {
        graph_edges.extend(edges);
    }

    let response = ApiResponse {
        success: true,
        data: Some(GraphData {
            nodes: graph_nodes,
            edges: graph_edges,
        }),
        error: None,
    };
    (StatusCode::OK, Json(response))
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "TAO Graph Database",
        "timestamp": current_time_millis()
    }))
}

async fn seed_data_handler() -> impl IntoResponse {
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

        match user_builder.savex().await {
            Ok(user) => {
                users.push(user);
            }
            Err(e) => {
                println!("Failed to create EntUser {}: {}", name, e);
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
                let to_user = &users[to_idx]; // Get the EntUser object, not just the ID
                let result = match rel_type {
                    "friendship" => from_user.add_friend(to_user.id).await,
                    "follows" => from_user.add_following(to_user.id).await,
                    _ => Ok(()), // Should not happen with defined types
                };

                match result {
                    Ok(_) => {
                        println!(
                            "Created {} relationship between {} and {}",
                            rel_type, from_user.id, to_user.id
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to create {} relationship between {} and {}: {}",
                            rel_type, from_user.id, to_user.id, e
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
    let shard_urls = [
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
        println!("✅ Shard {} configured", i + 1);
    }
    println!("✅ All shards configured");

    // Create TAO with WAL
    let association_registry = Arc::new(AssociationRegistry::new());

    // Setup WAL
    // let wal_config = WalConfig::default();
    // let wal = Arc::new(TaoWriteAheadLog::new(wal_config, "/tmp/tao_web_wal").await?);

    // Initialize cache and metrics
    // let cache = initialize_cache_default().await?;
    // let metrics = initialize_metrics_default().await?;

    // Create TaoCore instance
    let tao_core = Arc::new(
        tao_database::infrastructure::tao_core::tao_core::TaoCore::new(
            query_router.clone(),
            association_registry.clone(),
        ),
    );

    // Initialize TAO with all components
    let tao = Arc::new(Tao::minimal(tao_core));
    println!("✅ TAO initialized with production features");

    set_global_tao(tao).expect("Failed to set global TAO instance"); // Set global TAO

    // Application state
    let app_state = AppState {}; // Use global TAO

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/users", get(get_all_users).post(create_user))
        .route("/api/users/{id}", get(get_user))
        .route("/api/relationships", post(create_relationship))
        .route("/api/graph", get(get_graph_data))
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
