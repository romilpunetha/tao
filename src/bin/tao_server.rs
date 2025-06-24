use axum::{
    extract::{Path, Query},
    http::{header, Method, StatusCode},
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{info, warn};

use tao_database::domains::user::{EntUser, EntUserBuilder};
use tao_database::error::AppResult;
use tao_database::infrastructure::id_generator::get_id_generator;
use tao_database::ent_framework::Entity;

// API Response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

// Request/Response types
#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    full_name: Option<String>,
    bio: Option<String>,
    location: Option<String>,
}

#[derive(Deserialize)]
struct CreatePostRequest {
    author_id: i64,
    content: String,
    post_type: String,
    visibility: Option<String>,
    media_url: Option<String>,
}

#[derive(Deserialize)]
struct CreateFriendshipRequest {
    user1_id: i64,
    user2_id: i64,
    relationship_type: Option<String>,
}

#[derive(Deserialize)]
struct CreateFollowRequest {
    follower_id: i64,
    followee_id: i64,
    follow_type: Option<String>,
}

#[derive(Deserialize)]
struct CreateLikeRequest {
    user_id: i64,
    target_id: i64,
    reaction_type: String,
}

// Graph visualization types
#[derive(Serialize)]
struct GraphNode {
    id: String,
    name: String,
    node_type: String,
    verified: bool,
}

#[derive(Serialize)]
struct GraphEdge {
    source: String,
    target: String,
    edge_type: String,
    weight: f64,
}

#[derive(Serialize)]
struct GraphData {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Serialize)]
struct UserStats {
    user_id: i64,
    friend_count: i64,
    follower_count: i64,
    following_count: i64,
    post_count: i64,
}

// Query parameters
#[derive(Deserialize)]
struct LimitQuery {
    limit: Option<i64>,
    viewer_id: Option<i64>,
}

#[derive(Deserialize)]
struct GraphQuery {
    max_users: Option<i64>,
    viewer_id: Option<i64>,
}

// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("TAO Database Server is running!".to_string()))
}

// User endpoints
async fn create_user(Json(req): Json<CreateUserRequest>) -> Json<ApiResponse<EntUser>> {
    let mut builder = EntUserBuilder::new()
        .username(req.username)
        .email(req.email)
        .is_verified(false);
    
    if let Some(full_name) = req.full_name {
        builder = builder.full_name(full_name);
    }
    if let Some(bio) = req.bio {
        builder = builder.bio(bio);
    }
    if let Some(location) = req.location {
        builder = builder.location(location);
    }
    
    let user = builder.save().await;

    match user {
        Ok(user) => Json(ApiResponse::success(user)),
        Err(e) => {
            warn!("Failed to create user: {}", e);
            Json(ApiResponse::error(format!("Failed to create user: {}", e)))
        }
    }
}

async fn get_user(Path(user_id): Path<i64>) -> Json<ApiResponse<EntUser>> {
    match EntUser::gen_nullable(Some(user_id)).await {
        Ok(Some(user)) => Json(ApiResponse::success(user)),
        Ok(None) => Json(ApiResponse::error("User not found".to_string())),
        Err(e) => {
            warn!("Failed to get user {}: {}", user_id, e);
            Json(ApiResponse::error(format!("Failed to get user: {}", e)))
        }
    }
}

async fn get_all_users(Query(params): Query<LimitQuery>) -> Json<ApiResponse<Vec<EntUser>>> {
    // For now, we'll generate sample users since we don't have a complete database setup
    match generate_sample_users(params.limit.unwrap_or(10)).await {
        Ok(users) => Json(ApiResponse::success(users)),
        Err(e) => {
            warn!("Failed to generate sample users: {}", e);
            Json(ApiResponse::error(format!("Failed to get users: {}", e)))
        }
    }
}

async fn delete_user(Path(user_id): Path<i64>) -> Json<ApiResponse<String>> {
    // For demo purposes, just return success
    Json(ApiResponse::success(format!("User {} deleted", user_id)))
}

// Friend operations
async fn get_user_friends(
    Path(user_id): Path<i64>,
    Query(params): Query<LimitQuery>,
) -> Json<ApiResponse<Vec<EntUser>>> {
    match EntUser::gen_nullable(Some(user_id)).await {
        Ok(Some(user)) => {
            match user.get_friends().await {
                Ok(friends) => {
                    let limited_friends = if let Some(limit) = params.limit {
                        friends.into_iter().take(limit as usize).collect()
                    } else {
                        friends
                    };
                    Json(ApiResponse::success(limited_friends))
                }
                Err(e) => {
                    warn!("Failed to get friends for user {}: {}", user_id, e);
                    Json(ApiResponse::error(format!("Failed to get friends: {}", e)))
                }
            }
        }
        Ok(None) => Json(ApiResponse::error("User not found".to_string())),
        Err(e) => {
            warn!("Failed to get user {}: {}", user_id, e);
            Json(ApiResponse::error(format!("Failed to get user: {}", e)))
        }
    }
}

async fn get_user_stats(Path(user_id): Path<i64>) -> Json<ApiResponse<UserStats>> {
    match EntUser::gen_nullable(Some(user_id)).await {
        Ok(Some(user)) => {
            let friend_count = user.count_friends().await.unwrap_or(0);
            let follower_count = user.count_followers().await.unwrap_or(0);
            let following_count = user.count_following().await.unwrap_or(0);
            let post_count = user.count_posts().await.unwrap_or(0);

            let stats = UserStats {
                user_id,
                friend_count,
                follower_count,
                following_count,
                post_count,
            };

            Json(ApiResponse::success(stats))
        }
        Ok(None) => Json(ApiResponse::error("User not found".to_string())),
        Err(e) => {
            warn!("Failed to get user stats for {}: {}", user_id, e);
            Json(ApiResponse::error(format!("Failed to get user stats: {}", e)))
        }
    }
}

// Social graph operations
async fn create_friendship(Json(req): Json<CreateFriendshipRequest>) -> Json<ApiResponse<String>> {
    match (EntUser::gen_nullable(Some(req.user1_id)).await, EntUser::gen_nullable(Some(req.user2_id)).await) {
        (Ok(Some(user1)), Ok(Some(user2))) => {
            // Add bidirectional friendship
            if let (Ok(_), Ok(_)) = (user1.add_friend(req.user2_id).await, user2.add_friend(req.user1_id).await) {
                Json(ApiResponse::success("Friendship created successfully".to_string()))
            } else {
                Json(ApiResponse::error("Failed to create friendship".to_string()))
            }
        }
        _ => Json(ApiResponse::error("One or both users not found".to_string())),
    }
}

async fn create_follow(Json(req): Json<CreateFollowRequest>) -> Json<ApiResponse<String>> {
    match EntUser::gen_nullable(Some(req.follower_id)).await {
        Ok(Some(follower)) => {
            if let Ok(_) = follower.add_following(req.followee_id).await {
                Json(ApiResponse::success("Follow relationship created successfully".to_string()))
            } else {
                Json(ApiResponse::error("Failed to create follow relationship".to_string()))
            }
        }
        _ => Json(ApiResponse::error("Follower user not found".to_string())),
    }
}

async fn create_like(Json(_req): Json<CreateLikeRequest>) -> Json<ApiResponse<String>> {
    // For demo purposes, just return success
    Json(ApiResponse::success("Like created successfully".to_string()))
}

// Graph visualization endpoint
async fn get_graph_data(Query(params): Query<GraphQuery>) -> Json<ApiResponse<GraphData>> {
    let max_users = params.max_users.unwrap_or(20);
    
    match generate_graph_data(max_users).await {
        Ok(graph_data) => Json(ApiResponse::success(graph_data)),
        Err(e) => {
            warn!("Failed to generate graph data: {}", e);
            Json(ApiResponse::error(format!("Failed to generate graph data: {}", e)))
        }
    }
}

// Seed sample data
async fn seed_sample_data() -> Json<ApiResponse<String>> {
    match create_sample_data().await {
        Ok(_) => Json(ApiResponse::success("Sample data seeded successfully".to_string())),
        Err(e) => {
            warn!("Failed to seed sample data: {}", e);
            Json(ApiResponse::error(format!("Failed to seed sample data: {}", e)))
        }
    }
}

// Helper functions
async fn generate_sample_users(count: i64) -> AppResult<Vec<EntUser>> {
    let mut users = Vec::new();
    let sample_names = vec![
        ("alice", "Alice Johnson", "alice@example.com"),
        ("bob", "Bob Smith", "bob@example.com"),
        ("carol", "Carol Brown", "carol@example.com"),
        ("david", "David Wilson", "david@example.com"),
        ("eve", "Eve Davis", "eve@example.com"),
        ("frank", "Frank Miller", "frank@example.com"),
        ("grace", "Grace Taylor", "grace@example.com"),
        ("henry", "Henry Anderson", "henry@example.com"),
        ("ivy", "Ivy Thompson", "ivy@example.com"),
        ("jack", "Jack Moore", "jack@example.com"),
    ];

    for i in 0..count.min(sample_names.len() as i64) {
        let (username, full_name, email) = &sample_names[i as usize];
        let user = EntUserBuilder::new()
            .username(username.to_string())
            .email(email.to_string())
            .full_name(full_name.to_string())
            .bio(format!("Bio for {}", full_name))
            .is_verified(i % 3 == 0) // Every 3rd user is verified
            .build()
            .await?;
        users.push(user);
    }

    Ok(users)
}

async fn generate_graph_data(max_users: i64) -> AppResult<GraphData> {
    let users = generate_sample_users(max_users).await?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Create nodes
    for user in &users {
        nodes.push(GraphNode {
            id: user.id.to_string(),
            name: user.full_name.clone().unwrap_or_else(|| user.username.clone()),
            node_type: "user".to_string(),
            verified: user.is_verified,
        });
    }

    // Create sample edges (friendships and follows)
    for i in 0..users.len() {
        for j in (i + 1)..users.len() {
            // Create some friendships (about 30% chance)
            if (i + j) % 3 == 0 {
                edges.push(GraphEdge {
                    source: users[i].id.to_string(),
                    target: users[j].id.to_string(),
                    edge_type: "friendship".to_string(),
                    weight: 1.0,
                });
            }
            // Create some follow relationships (about 20% chance)
            else if (i + j) % 5 == 0 {
                edges.push(GraphEdge {
                    source: users[i].id.to_string(),
                    target: users[j].id.to_string(),
                    edge_type: "follow".to_string(),
                    weight: 0.8,
                });
            }
        }
    }

    Ok(GraphData { nodes, edges })
}

async fn create_sample_data() -> AppResult<()> {
    let users = generate_sample_users(10).await?;
    
    // Create some friendships between users
    for i in 0..users.len() {
        for j in (i + 1)..users.len() {
            if (i + j) % 3 == 0 {
                let _ = users[i].add_friend(users[j].id).await;
                let _ = users[j].add_friend(users[i].id).await;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting TAO Database Server...");

    // Build the router
    let app = Router::new()
        // Health check
        .route("/api/health", get(health_check))
        // User operations
        .route("/api/users", get(get_all_users).post(create_user))
        .route("/api/users/:id", get(get_user).delete(delete_user))
        .route("/api/users/:id/friends", get(get_user_friends))
        .route("/api/users/:id/stats", get(get_user_stats))
        // Social graph operations
        .route("/api/friendships", post(create_friendship))
        .route("/api/follows", post(create_follow))
        .route("/api/likes", post(create_like))
        // Graph visualization
        .route("/api/graph", get(get_graph_data))
        // Utility
        .route("/api/seed", post(seed_sample_data))
        // Serve static files
        .nest_service("/", ServeDir::new("frontend/build"))
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods([Method::GET, Method::POST, Method::DELETE])
                        .allow_headers([header::CONTENT_TYPE]),
                )
        );

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;
    info!("🌐 Server listening on http://127.0.0.1:8000");
    info!("📊 API available at http://127.0.0.1:8000/api");
    info!("🎨 Frontend available at http://127.0.0.1:8000");

    axum::serve(listener, app).await?;

    Ok(())
}