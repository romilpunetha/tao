import { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { TaoApiService } from '../services/api';
import {
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

// Query keys
export const QUERY_KEYS = {
  HEALTH: 'health',
  USERS: 'users',
  USER: 'user',
  USER_POSTS: 'userPosts',
  USER_FRIENDS: 'userFriends',
  USER_STATS: 'userStats',
  GRAPH_DATA: 'graphData',
} as const;

// Health check hook
export function useHealthCheck() {
  return useQuery({
    queryKey: [QUERY_KEYS.HEALTH],
    queryFn: TaoApiService.healthCheck,
    staleTime: 30000, // 30 seconds
    refetchInterval: 60000, // 1 minute
  });
}

// User hooks
export function useUsers(limit?: number) {
  return useQuery({
    queryKey: [QUERY_KEYS.USERS, limit],
    queryFn: () => TaoApiService.getAllUsers(limit),
    staleTime: 10000, // 10 seconds
  });
}

export function useUser(userId: number) {
  return useQuery({
    queryKey: [QUERY_KEYS.USER, userId],
    queryFn: () => TaoApiService.getUser(userId),
    enabled: !!userId,
    staleTime: 30000,
  });
}

export function useUserPosts(userId: number, limit?: number, viewerId?: number) {
  return useQuery({
    queryKey: [QUERY_KEYS.USER_POSTS, userId, limit, viewerId],
    queryFn: () => TaoApiService.getUserPosts(userId, limit, viewerId),
    enabled: !!userId,
    staleTime: 10000,
  });
}

export function useUserFriends(userId: number, limit?: number, viewerId?: number) {
  return useQuery({
    queryKey: [QUERY_KEYS.USER_FRIENDS, userId, limit, viewerId],
    queryFn: () => TaoApiService.getUserFriends(userId, limit, viewerId),
    enabled: !!userId,
    staleTime: 10000,
  });
}

export function useUserStats(userId: number) {
  return useQuery({
    queryKey: [QUERY_KEYS.USER_STATS, userId],
    queryFn: () => TaoApiService.getUserStats(userId),
    enabled: !!userId,
    staleTime: 30000,
  });
}

export function useGraphData(maxUsers?: number, viewerId?: number) {
  return useQuery({
    queryKey: [QUERY_KEYS.GRAPH_DATA, maxUsers, viewerId],
    queryFn: () => TaoApiService.getGraphData(maxUsers, viewerId),
    staleTime: 5000, // 5 seconds - graph data changes frequently
    refetchOnWindowFocus: false,
  });
}

// Mutation hooks
export function useCreateUser() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (userData: CreateUserRequest) => TaoApiService.createUser(userData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USERS] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.GRAPH_DATA] });
    },
  });
}

export function useDeleteUser() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (userId: number) => TaoApiService.deleteUser(userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USERS] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.GRAPH_DATA] });
    },
  });
}

export function useCreatePost() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (postData: CreatePostRequest) => TaoApiService.createPost(postData),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_POSTS, variables.author_id] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_STATS, variables.author_id] });
    },
  });
}

export function useCreateFriendship() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (friendshipData: CreateFriendshipRequest) => TaoApiService.createFriendship(friendshipData),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_FRIENDS, variables.user1_id] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_FRIENDS, variables.user2_id] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_STATS, variables.user1_id] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_STATS, variables.user2_id] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.GRAPH_DATA] });
    },
  });
}

export function useCreateFollow() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (followData: CreateFollowRequest) => TaoApiService.createFollow(followData),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_STATS, variables.follower_id] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_STATS, variables.followee_id] });
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.GRAPH_DATA] });
    },
  });
}

export function useCreateLike() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (likeData: CreateLikeRequest) => TaoApiService.createLike(likeData),
    onSuccess: () => {
      // Invalidate relevant queries - in a real app you'd be more specific
      queryClient.invalidateQueries({ queryKey: [QUERY_KEYS.USER_POSTS] });
    },
  });
}

export function useSeedSampleData() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: () => TaoApiService.seedSampleData(),
    onSuccess: () => {
      // Invalidate all queries to refresh data
      queryClient.invalidateQueries();
    },
  });
}

// Custom hook for managing selected user
export function useSelectedUser() {
  const [selectedUserId, setSelectedUserId] = useState<number | null>(null);
  
  const user = useUser(selectedUserId || 0);
  const posts = useUserPosts(selectedUserId || 0);
  const friends = useUserFriends(selectedUserId || 0);
  const stats = useUserStats(selectedUserId || 0);
  
  return {
    selectedUserId,
    setSelectedUserId,
    user: user.data,
    posts: posts.data || [],
    friends: friends.data || [],
    stats: stats.data,
    isLoading: user.isPending || posts.isPending || friends.isPending || stats.isPending,
    error: user.error || posts.error || friends.error || stats.error,
  };
}