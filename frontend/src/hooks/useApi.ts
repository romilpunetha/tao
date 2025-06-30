import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { TaoApiService } from '../services/api';
import { CreateUserRequest, CreatePostRequest, CreateFriendshipRequest, CreateFollowRequest, CreateLikeRequest } from '../types/api';

const STALE_TIME = 1000 * 60 * 5; // 5 minutes

// Health Check
export const useHealthCheck = () => {
  return useQuery({
    queryKey: ['health'],
    queryFn: TaoApiService.healthCheck,
    staleTime: STALE_TIME,
  });
};

// Users
export const useUsers = (limit?: number) => {
  return useQuery({
    queryKey: ['users', limit],
    queryFn: () => TaoApiService.getAllUsers(limit),
    staleTime: STALE_TIME,
  });
};

export const useUser = (userId: number) => {
  return useQuery({
    queryKey: ['user', userId],
    queryFn: () => TaoApiService.getUser(userId),
    enabled: !!userId,
    staleTime: STALE_TIME,
  });
};

export const useCreateUser = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (userData: CreateUserRequest) => TaoApiService.createUser(userData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};

export const useDeleteUser = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (userId: number) => TaoApiService.deleteUser(userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};

// Posts
export const useUserPosts = (userId: number, limit?: number, viewerId?: number) => {
  return useQuery({
    queryKey: ['posts', userId, limit, viewerId],
    queryFn: () => TaoApiService.getUserPosts(userId, limit, viewerId),
    enabled: !!userId,
    staleTime: STALE_TIME,
  });
};

export const useCreatePost = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (postData: CreatePostRequest) => TaoApiService.createPost(postData),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['posts', data.author_id] });
    },
  });
};

// Social Graph
export const useCreateFriendship = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (friendshipData: CreateFriendshipRequest) => TaoApiService.createFriendship(friendshipData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['graph'] });
      queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};

export const useCreateFollow = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (followData: CreateFollowRequest) => TaoApiService.createFollow(followData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['graph'] });
    },
  });
};

export const useCreateLike = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (likeData: CreateLikeRequest) => TaoApiService.createLike(likeData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['posts'] });
    },
  });
};

export const useUserFriends = (userId: number, limit?: number, viewerId?: number) => {
  return useQuery({
    queryKey: ['friends', userId, limit, viewerId],
    queryFn: () => TaoApiService.getUserFriends(userId, limit, viewerId),
    enabled: !!userId,
    staleTime: STALE_TIME,
  });
};

export const useUserStats = (userId: number) => {
  return useQuery({
    queryKey: ['stats', userId],
    queryFn: () => TaoApiService.getUserStats(userId),
    enabled: !!userId,
    staleTime: STALE_TIME,
  });
};

// Graph Data
export const useGraphData = (maxUsers?: number, viewerId?: number) => {
  return useQuery({
    queryKey: ['graph', maxUsers, viewerId],
    queryFn: () => TaoApiService.getGraphData(maxUsers, viewerId),
    staleTime: STALE_TIME,
  });
};

// Utility
export const useSeedSampleData = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => TaoApiService.seedSampleData(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['graph'] });
      queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};
