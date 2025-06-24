use axum::{
    http::{header, Method},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;

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

// Sample user data
#[derive(Serialize)]
struct User {
    id: i64,
    username: String,
    email: String,
    full_name: Option<String>,
    bio: Option<String>,
    created_time: i64,
    is_verified: bool,
}

// Health check endpoint
async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("TAO Database Server is running!".to_string()))
}

// Generate sample graph data
async fn get_graph_data() -> Json<ApiResponse<GraphData>> {
    let sample_users = vec![
        ("1", "Alice Johnson", "alice", true),
        ("2", "Bob Smith", "bob", false),
        ("3", "Carol Brown", "carol", false),
        ("4", "David Wilson", "david", true),
        ("5", "Eve Davis", "eve", false),
        ("6", "Frank Miller", "frank", false),
        ("7", "Grace Taylor", "grace", true),
        ("8", "Henry Anderson", "henry", false),
        ("9", "Ivy Thompson", "ivy", false),
        ("10", "Jack Moore", "jack", false),
    ];

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Create nodes
    for (id, name, username, verified) in &sample_users {
        nodes.push(GraphNode {
            id: id.to_string(),
            name: name.to_string(),
            node_type: "user".to_string(),
            verified: *verified,
        });
    }

    // Create sample edges (friendships and follows)
    let friendships = vec![
        ("1", "2"), ("1", "3"), ("2", "4"), ("3", "5"), 
        ("4", "6"), ("5", "7"), ("6", "8"), ("7", "9"), ("8", "10")
    ];
    
    let follows = vec![
        ("2", "1"), ("3", "1"), ("5", "4"), ("6", "4"),
        ("8", "7"), ("9", "7"), ("10", "9")
    ];

    // Add friendship edges
    for (source, target) in friendships {
        edges.push(GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            edge_type: "friendship".to_string(),
            weight: 1.0,
        });
    }

    // Add follow edges
    for (source, target) in follows {
        edges.push(GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            edge_type: "follow".to_string(),
            weight: 0.8,
        });
    }

    Json(ApiResponse::success(GraphData { nodes, edges }))
}

// Get sample users
async fn get_users() -> Json<ApiResponse<Vec<User>>> {
    let sample_users = vec![
        User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            full_name: Some("Alice Johnson".to_string()),
            bio: Some("Software engineer and tech enthusiast".to_string()),
            created_time: 1640995200000, // 2022-01-01
            is_verified: true,
        },
        User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            full_name: Some("Bob Smith".to_string()),
            bio: Some("Designer and artist".to_string()),
            created_time: 1641081600000, // 2022-01-02
            is_verified: false,
        },
        User {
            id: 3,
            username: "carol".to_string(),
            email: "carol@example.com".to_string(),
            full_name: Some("Carol Brown".to_string()),
            bio: Some("Product manager".to_string()),
            created_time: 1641168000000, // 2022-01-03
            is_verified: false,
        },
        User {
            id: 4,
            username: "david".to_string(),
            email: "david@example.com".to_string(),
            full_name: Some("David Wilson".to_string()),
            bio: Some("Data scientist".to_string()),
            created_time: 1641254400000, // 2022-01-04
            is_verified: true,
        },
        User {
            id: 5,
            username: "eve".to_string(),
            email: "eve@example.com".to_string(),
            full_name: Some("Eve Davis".to_string()),
            bio: Some("Marketing specialist".to_string()),
            created_time: 1641340800000, // 2022-01-05
            is_verified: false,
        },
    ];

    Json(ApiResponse::success(sample_users))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting TAO Database Demo Server...");

    // Build the router
    let app = Router::new()
        // Health check
        .route("/api/health", get(health_check))
        // Demo endpoints
        .route("/api/users", get(get_users))
        .route("/api/graph", get(get_graph_data))
        // Serve static files as fallback
        .fallback_service(ServeDir::new("frontend/build"))
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods([Method::GET, Method::POST])
                        .allow_headers([header::CONTENT_TYPE]),
                )
        );

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;
    info!("🌐 Server listening on http://127.0.0.1:8000");
    info!("📊 API available at http://127.0.0.1:8000/api");
    info!("🎨 Frontend available at http://127.0.0.1:8000");
    info!("📈 Graph visualization: http://127.0.0.1:8000/api/graph");

    axum::serve(listener, app).await?;

    Ok(())
}