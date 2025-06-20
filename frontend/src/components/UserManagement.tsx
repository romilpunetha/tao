import React, { useState } from 'react';
import {
  Box,
  Paper,
  Typography,
  TextField,
  Button,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Chip,
  Avatar,
  Alert,
  CircularProgress,
  Divider,
} from '@mui/material';
import {
  Add as AddIcon,
  Delete as DeleteIcon,
  Person as PersonIcon,
  Verified as VerifiedIcon,
  Group as GroupIcon,
  Article as ArticleIcon,
} from '@mui/icons-material';
import { useUsers, useCreateUser, useDeleteUser, useUserStats } from '../hooks/useApi';
import { CreateUserRequest, User, UserStats } from '../types/api';

interface UserManagementProps {
  selectedUserId?: number | null;
  onUserSelect?: (userId: number) => void;
}

export const UserManagement: React.FC<UserManagementProps> = ({
  selectedUserId,
  onUserSelect,
}) => {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [newUser, setNewUser] = useState<CreateUserRequest>({
    username: '',
    email: '',
    full_name: '',
    bio: '',
    location: '',
  });

  // API hooks
  const { data: users = [], isLoading, error, refetch } = useUsers();
  const createUserMutation = useCreateUser();
  const deleteUserMutation = useDeleteUser();

  const handleCreateUser = async () => {
    if (!newUser.username || !newUser.email) return;

    try {
      await createUserMutation.mutateAsync(newUser);
      setCreateDialogOpen(false);
      setNewUser({
        username: '',
        email: '',
        full_name: '',
        bio: '',
        location: '',
      });
      refetch();
    } catch (error) {
      console.error('Failed to create user:', error);
    }
  };

  const handleDeleteUser = async (userId: number) => {
    if (window.confirm('Are you sure you want to delete this user?')) {
      try {
        await deleteUserMutation.mutateAsync(userId);
        refetch();
      } catch (error) {
        console.error('Failed to delete user:', error);
      }
    }
  };

  const UserStatsDisplay: React.FC<{ userId: number }> = ({ userId }) => {
    const { data: stats } = useUserStats(userId);
    
    if (!stats) return null;

    return (
      <Box sx={{ display: 'flex', gap: 1, mt: 1 }}>
        <Chip
          size="small"
          icon={<GroupIcon />}
          label={`${stats.friend_count} friends`}
          variant="outlined"
        />
        <Chip
          size="small"
          icon={<ArticleIcon />}
          label={`${stats.post_count} posts`}
          variant="outlined"
        />
      </Box>
    );
  };

  if (error) {
    return (
      <Paper sx={{ p: 3 }}>
        <Alert severity="error">
          Failed to load users: {error instanceof Error ? error.message : 'Unknown error'}
        </Alert>
      </Paper>
    );
  }

  return (
    <Paper sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Typography variant="h6">
            Users ({users.length})
          </Typography>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => setCreateDialogOpen(true)}
            size="small"
          >
            Add User
          </Button>
        </Box>
      </Box>

      {/* User List */}
      <Box sx={{ flex: 1, overflow: 'auto' }}>
        {isLoading ? (
          <Box sx={{ display: 'flex', justifyContent: 'center', p: 3 }}>
            <CircularProgress />
          </Box>
        ) : (
          <List sx={{ p: 0 }}>
            {users.map((user, index) => (
              <React.Fragment key={user.id}>
                <ListItem
                  sx={{
                    cursor: 'pointer',
                    backgroundColor: selectedUserId === user.id ? 'action.selected' : 'transparent',
                    '&:hover': {
                      backgroundColor: 'action.hover',
                    },
                  }}
                  onClick={() => onUserSelect?.(user.id)}
                >
                  <Avatar sx={{ mr: 2, bgcolor: user.is_verified ? 'primary.main' : 'grey.400' }}>
                    {user.is_verified ? <VerifiedIcon /> : <PersonIcon />}
                  </Avatar>
                  
                  <ListItemText
                    primary={
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <Typography variant="subtitle2">
                          {user.full_name || user.username}
                        </Typography>
                        {user.is_verified && (
                          <VerifiedIcon color="primary" sx={{ fontSize: 16 }} />
                        )}
                      </Box>
                    }
                    secondary={
                      <Box>
                        <Typography variant="body2" color="text.secondary">
                          @{user.username} • {user.email}
                        </Typography>
                        {user.bio && (
                          <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 0.5 }}>
                            {user.bio}
                          </Typography>
                        )}
                        {user.location && (
                          <Typography variant="caption" color="text.secondary">
                            📍 {user.location}
                          </Typography>
                        )}
                        <UserStatsDisplay userId={user.id} />
                      </Box>
                    }
                  />
                  
                  <ListItemSecondaryAction>
                    <IconButton
                      edge="end"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteUser(user.id);
                      }}
                      disabled={deleteUserMutation.isPending}
                      size="small"
                    >
                      <DeleteIcon />
                    </IconButton>
                  </ListItemSecondaryAction>
                </ListItem>
                {index < users.length - 1 && <Divider />}
              </React.Fragment>
            ))}
          </List>
        )}
      </Box>

      {/* Create User Dialog */}
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create New User</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
            <TextField
              label="Username"
              value={newUser.username}
              onChange={(e) => setNewUser({ ...newUser, username: e.target.value })}
              required
              fullWidth
            />
            <TextField
              label="Email"
              type="email"
              value={newUser.email}
              onChange={(e) => setNewUser({ ...newUser, email: e.target.value })}
              required
              fullWidth
            />
            <TextField
              label="Full Name"
              value={newUser.full_name}
              onChange={(e) => setNewUser({ ...newUser, full_name: e.target.value })}
              fullWidth
            />
            <TextField
              label="Bio"
              value={newUser.bio}
              onChange={(e) => setNewUser({ ...newUser, bio: e.target.value })}
              multiline
              rows={3}
              fullWidth
            />
            <TextField
              label="Location"
              value={newUser.location}
              onChange={(e) => setNewUser({ ...newUser, location: e.target.value })}
              fullWidth
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleCreateUser}
            variant="contained"
            disabled={!newUser.username || !newUser.email || createUserMutation.isPending}
          >
            {createUserMutation.isPending ? <CircularProgress size={20} /> : 'Create'}
          </Button>
        </DialogActions>
      </Dialog>
    </Paper>
  );
};