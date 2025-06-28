use serde_json::json;
use std::sync::Arc;
use crate::{
    error::AppResult, 
    infrastructure::{tao::Tao, tao_core::{create_tao_association, current_time_millis}, TaoOperations},
    domains::user::EntUser,
    domains::post::EntPost,
    domains::comment::EntComment,
};

pub async fn seed_data_into_tao(tao: Arc<Tao>) -> AppResult<()> {
    // Generate sample users
    let sample_users = vec![
        ("Alice Johnson", "alice@example.com", "Software Engineer at Meta"),
        ("Bob Smith", "bob@example.com", "Product Manager"),
        ("Carol Wilson", "carol@example.com", "UX Designer"),
        ("David Brown", "david@example.com", "Data Scientist"),
        ("Eve Davis", "eve@example.com", "DevOps Engineer"),
        ("Frank Miller", "frank@example.com", "Mobile Developer"),
        ("Grace Lee", "grace@example.com", "Backend Engineer"),
        ("Henry Taylor", "henry@example.com", "Frontend Developer"),
        ("Ivy Chen", "ivy@example.com", "Machine Learning Engineer"),
        ("Jack Wilson", "jack@example.com", "Security Engineer"),
    ];

    let mut created_user_ids = Vec::new();
    
    println!("\n👥 Creating {} users...", sample_users.len());
    for (name, email, bio) in sample_users {
        let username = name.to_lowercase().replace(" ", "_");
        
        let user = EntUser::create()
            .username(username.clone())
            .email(email.to_string())
            .full_name(name.to_string())
            .bio(bio.to_string())
            .profile_picture_url(format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", name.replace(" ", "")))
            .location("San Francisco, CA".to_string())
            .is_verified(true)
            .privacy_settings("public".to_string())
            .save()
            .await?;
        
        created_user_ids.push(user.id);
        println!("  ✓ Created user '{}' with ID: {}", name, user.id);
    }

    // Generate sample relationships
    println!("\n🤝 Creating relationships...");
    let relationships = vec![
        // Friendships (bidirectional)
        (0, 1, "friendship"), (1, 0, "friendship"),
        (1, 2, "friendship"), (2, 1, "friendship"),
        (2, 3, "friendship"), (3, 2, "friendship"),
        (3, 4, "friendship"), (4, 3, "friendship"),
        (4, 5, "friendship"), (5, 4, "friendship"),
        (0, 5, "friendship"), (5, 0, "friendship"),
        (0, 2, "friendship"), (2, 0, "friendship"),
        (1, 4, "friendship"), (4, 1, "friendship"),
        
        // Follows (unidirectional)
        (0, 6, "follows"), (1, 6, "follows"), (2, 6, "follows"),
        (6, 7, "follows"), (7, 8, "follows"), (8, 9, "follows"),
        (3, 7, "follows"), (4, 8, "follows"), (5, 9, "follows"),
        (0, 9, "follows"), (1, 8, "follows"), (2, 7, "follows"),
        
        // Work relationships
        (0, 1, "colleague"), (1, 2, "colleague"), (2, 3, "colleague"),
        (6, 7, "teammate"), (7, 8, "teammate"), (8, 9, "teammate"),
        
        // Mentorship
        (6, 0, "mentor"), (7, 1, "mentor"), (8, 2, "mentor"), (9, 3, "mentor"),
    ];

    for (from_idx, to_idx, relationship_type) in &relationships {
        if *from_idx < created_user_ids.len() && *to_idx < created_user_ids.len() {
            let from_id = created_user_ids[*from_idx];
            let to_id = created_user_ids[*to_idx];
            
            let association = create_tao_association(
                from_id,
                relationship_type.to_string(),
                to_id,
                Some(json!({
                    "created_at": current_time_millis(),
                    "strength": (0.5 + (*from_idx + *to_idx) as f64 * 0.1) % 1.0,
                    "context": match *relationship_type {
                        "friendship" => "Met through mutual friends",
                        "follows" => "Professional interest",
                        "colleague" => "Work in the same team",
                        "teammate" => "Collaborate on projects",
                        "mentor" => "Professional mentorship",
                        _ => "Other"
                    }
                }).to_string().into_bytes()),
            );
            
            tao.assoc_add(association).await?;
            println!("  ✓ Created {} relationship: {} -> {}", relationship_type, from_id, to_id);
        }
    }

    // Generate some sample posts/content
    println!("\n📝 Creating sample posts...");
    let sample_posts = vec![
        ("Alice Johnson", "Just shipped a new feature for TAO! 🚀 #engineering #meta"),
        ("Bob Smith", "Product roadmap planning session today. Exciting features coming! #product"),
        ("Carol Wilson", "User research insights are driving our next design iteration 🎨 #ux"),
        ("David Brown", "ML model performance improved by 23% after optimization! 📊 #datascience"),
        ("Eve Davis", "Kubernetes deployment went smoothly. Infrastructure is scaling! ⚙️ #devops"),
        ("Frank Miller", "New mobile app features are looking great on iOS 📱 #mobile"),
        ("Grace Lee", "Backend API now handles 1M+ requests per second 💪 #backend"),
        ("Henry Taylor", "React performance optimization reduced bundle size by 40% ⚡ #frontend"),
    ];

    let mut created_post_ids = Vec::new();
    for (i, (author, content)) in sample_posts.iter().enumerate() {
        if i < created_user_ids.len() {
            let author_id = created_user_ids[i];
            let post_data = json!({
                "content": content,
                "author_id": author_id,
                "author_name": author,
                "created_at": current_time_millis() - (i as i64 * 3600000), // Spread posts over time
                "likes_count": (i * 7 + 3) % 50,
                "comments_count": (i * 3 + 1) % 15,
                "hashtags": content.split("#").skip(1).map(|tag| tag.trim().split(" ").next().unwrap_or("")).collect::<Vec<_>>(),
                "visibility": "public"
            });
            
            let post_id = tao.obj_add("post".to_string(), post_data.to_string().into_bytes(), Some(author_id)).await?;
            created_post_ids.push(post_id);
            
            // Create authored relationship
            let authored_assoc = create_tao_association(
                author_id,
                "authored".to_string(),
                post_id,
                Some(json!({"created_at": current_time_millis()}).to_string().into_bytes()),
            );
            tao.assoc_add(authored_assoc).await?;
            
            println!("  ✓ Created post by {} (ID: {})", author, post_id);
        }
    }

    // Generate some likes and comments relationships
    println!("\n❤️ Creating likes and comments...");
    for (i, post_id) in created_post_ids.iter().enumerate() {
        // Some users like this post
        for j in 0..((i * 3 + 2) % 6) {
            if j < created_user_ids.len() && j != i {
                let user_id = created_user_ids[j];
                let like_assoc = create_tao_association(
                    user_id,
                    "likes".to_string(),
                    *post_id,
                    Some(json!({"created_at": current_time_millis()}).to_string().into_bytes()),
                );
                tao.assoc_add(like_assoc).await?;
            }
        }
        
        // Some users comment on this post
        for j in 0..((i * 2 + 1) % 4) {
            if j < created_user_ids.len() && j != i {
                let comment_data = json!({
                    "content": format!("Great post! This is really insightful. #{}", j),
                    "author_id": created_user_ids[j],
                    "post_id": post_id,
                    "created_at": current_time_millis() - (j as i64 * 1800000),
                });
                
                let comment_id = tao.obj_add("comment".to_string(), comment_data.to_string().into_bytes(), Some(created_user_ids[j])).await?;
                
                // Create comment relationship
                let comment_assoc = create_tao_association(
                    comment_id,
                    "comments_on".to_string(),
                    *post_id,
                    Some(json!({"created_at": current_time_millis()}).to_string().into_bytes()),
                );
                tao.assoc_add(comment_assoc).await?;
            }
        }
    }

    println!("\n🎯 Sample data generation complete!");
    println!("📊 Statistics:");
    println!("  - Users created: {}", created_user_ids.len());
    println!("  - Posts created: {}", created_post_ids.len());
    println!("  - Relationships: {} total", relationships.len());
    println!("  - User ID range: {} - {}", 
             created_user_ids.iter().min().unwrap_or(&0),
             created_user_ids.iter().max().unwrap_or(&0));
    
    Ok(())
}