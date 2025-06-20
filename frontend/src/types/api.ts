// API Response wrapper
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// Core entity types
export interface User {
  id: number;
  username: string;
  email: string;
  full_name?: string;
  bio?: string;
  profile_picture_url?: string;
  created_time: number;
  last_active_time?: number;
  is_verified: boolean;
  location?: string;
}

export interface Post {
  id: number;
  author_id: number;
  content: string;
  media_url?: string;
  created_time: number;
  updated_time?: number;
  post_type: string;
  visibility?: string;
  like_count: number;
  comment_count: number;
  share_count: number;
}

// Request types
export interface CreateUserRequest {
  username: string;
  email: string;
  full_name?: string;
  bio?: string;
  location?: string;
}

export interface CreatePostRequest {
  author_id: number;
  content: string;
  post_type: string;
  visibility?: string;
  media_url?: string;
}

export interface CreateFriendshipRequest {
  user1_id: number;
  user2_id: number;
  relationship_type?: string;
}

export interface CreateFollowRequest {
  follower_id: number;
  followee_id: number;
  follow_type?: string;
}

export interface CreateLikeRequest {
  user_id: number;
  target_id: number;
  reaction_type: string;
}

// Graph visualization types
export interface GraphNode {
  id: string;
  name: string;
  node_type: string;
  verified: boolean;
}

export interface GraphEdge {
  source: string;
  target: string;
  edge_type: string;
  weight: number;
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

// User statistics
export interface UserStats {
  user_id: number;
  friend_count: number;
  follower_count: number;
  following_count: number;
  post_count: number;
}

// D3 specific types for visualization
export interface D3Node extends GraphNode {
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
  index?: number;
}

export interface D3Edge extends Omit<GraphEdge, 'source' | 'target'> {
  source: D3Node | string;
  target: D3Node | string;
  index?: number;
}