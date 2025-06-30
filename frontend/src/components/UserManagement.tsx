import React from 'react';
import {
  Box,
  Paper,
  Typography,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  ListItemIcon,
  Avatar,
  CircularProgress,
  Alert,
  Divider,
} from '@mui/material';
import { Person as PersonIcon } from '@mui/icons-material';
import { useUsers } from '../hooks/useApi';

interface UserManagementProps {
  selectedUserId: number | null;
  onUserSelect: (userId: number) => void;
}

export const UserManagement: React.FC<UserManagementProps> = ({
  selectedUserId,
  onUserSelect,
}) => {
  const { data: users = [], isLoading, error } = useUsers();

  if (isLoading) {
    return (
      <Paper sx={{ p: 2, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress />
      </Paper>
    );
  }

  if (error) {
    return <Alert severity="error">Failed to load users</Alert>;
  }

  return (
    <Paper sx={{ height: '100%', overflow: 'auto' }}>
      <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        <Typography variant="h6">Users</Typography>
      </Box>
      <List dense>
        {users.map((user) => (
          <React.Fragment key={user.id}>
            <ListItem disablePadding>
              <ListItemButton
                selected={selectedUserId === user.id}
                onClick={() => onUserSelect(user.id)}
              >
                <ListItemIcon>
                  <Avatar sx={{ width: 32, height: 32 }}>
                    <PersonIcon />
                  </Avatar>
                </ListItemIcon>
                <ListItemText
                  primary={user.full_name || user.username}
                  secondary={user.email}
                />
              </ListItemButton>
            </ListItem>
            <Divider variant="inset" component="li" />
          </React.Fragment>
        ))}
      </List>
    </Paper>
  );
};
