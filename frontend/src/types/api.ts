export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface User {
  id: number;
  username: string;
  email: string;
  full_name?: string;
  bio?: string;
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
  tags?: string;
  mentions?: string;
}

export interface CreateUserRequest {
  name: string;
  email: string;
  bio?: string;
}

export interface CreatePostRequest {
  author_id: number;
  content: string;
  media_url?: string;
}

export interface CreateFriendshipRequest {
  from_user_id: number;
  to_user_id: number;
}

export interface CreateFollowRequest {
  from_user_id: number;
  to_user_id: number;
}

export interface CreateLikeRequest {
  user_id: number;
  post_id: number;
}

export interface UserStats {
  friend_count: number;
  follower_count: number;
  following_count: number;
  like_count: number;
  post_count: number;
}

export interface D3Node extends d3.SimulationNodeDatum {
  id: string;
  name: string;
  node_type: string;
  verified: boolean;
}

export interface D3Edge extends d3.SimulationLinkDatum<D3Node> {
  source: string | D3Node;
  target: string | D3Node;
  edge_type: string;
  weight: number;
}

export interface GraphData {
  nodes: D3Node[];
  edges: D3Edge[];
}
