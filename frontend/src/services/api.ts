import axios from 'axios';
import {
  ApiResponse,
  User,
  Post,
  CreateUserRequest,
  CreatePostRequest,
  CreateFriendshipRequest,
  CreateFollowRequest,
  CreateLikeRequest,
  GraphData,
  UserStats,
} from '../types/api';

const API_BASE_URL = 'http://localhost:3000/api';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add response interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Error:', error);
    return Promise.reject(error);
  }
);

export class TaoApiService {
  // Health check
  static async healthCheck(): Promise<string> {
    const response = await api.get<ApiResponse<string>>('/health');
    return response.data.data || 'Unknown status';
  }

  // User operations
  static async createUser(userData: CreateUserRequest): Promise<User> {
    const response = await api.post<ApiResponse<User>>('/users', userData);
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create user');
    }
    return response.data.data!;
  }

  static async getUser(userId: number): Promise<User> {
    const response = await api.get<ApiResponse<User>>(`/users/${userId}`);
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get user');
    }
    return response.data.data!;
  }

  static async getAllUsers(limit?: number): Promise<User[]> {
    const params = limit ? { limit } : {};
    const response = await api.get<ApiResponse<User[]>>('/users', { params });
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get users');
    }
    return response.data.data || [];
  }

  static async deleteUser(userId: number): Promise<string> {
    const response = await api.delete<ApiResponse<string>>(`/users/${userId}`);
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to delete user');
    }
    return response.data.data!;
  }

  // Post operations
  static async createPost(postData: CreatePostRequest): Promise<Post> {
    const response = await api.post<ApiResponse<Post>>('/posts', postData);
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create post');
    }
    return response.data.data!;
  }

  static async getUserPosts(userId: number, limit?: number, viewerId?: number): Promise<Post[]> {
    const params: any = {};
    if (limit) params.limit = limit;
    if (viewerId) params.viewer_id = viewerId;

    const response = await api.get<ApiResponse<Post[]>>(`/users/${userId}/posts`, { params });
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get user posts');
    }
    return response.data.data || [];
  }

  // Social graph operations
  static async createFriendship(friendshipData: CreateFriendshipRequest): Promise<string> {
    const response = await api.post<ApiResponse<string>>('/friendships', friendshipData);
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create friendship');
    }
    return response.data.data!;
  }

  static async createFollow(followData: CreateFollowRequest): Promise<string> {
    const response = await api.post<ApiResponse<string>>('/follows', followData);
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create follow');
    }
    return response.data.data!;
  }

  static async createLike(likeData: CreateLikeRequest): Promise<string> {
    const response = await api.post<ApiResponse<string>>('/likes', likeData);
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create like');
    }
    return response.data.data!;
  }

  static async getUserFriends(userId: number, limit?: number, viewerId?: number): Promise<User[]> {
    const params: any = {};
    if (limit) params.limit = limit;
    if (viewerId) params.viewer_id = viewerId;

    const response = await api.get<ApiResponse<User[]>>(`/users/${userId}/friends`, { params });
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get user friends');
    }
    return response.data.data || [];
  }

  static async getUserStats(userId: number): Promise<UserStats> {
    const response = await api.get<ApiResponse<UserStats>>(`/users/${userId}/stats`);
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get user stats');
    }
    return response.data.data!;
  }

  // Graph visualization
  static async getGraphData(maxUsers?: number, viewerId?: number): Promise<GraphData> {
    const params: any = {};
    if (maxUsers) params.max_users = maxUsers;
    if (viewerId) params.viewer_id = viewerId;

    const response = await api.get<ApiResponse<GraphData>>('/graph', { params });
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get graph data');
    }
    return response.data.data!;
  }

  // Utility
  static async seedSampleData(): Promise<string> {
    const response = await api.post<ApiResponse<string>>('/seed');
    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to seed sample data');
    }
    return response.data.data!;
  }
}