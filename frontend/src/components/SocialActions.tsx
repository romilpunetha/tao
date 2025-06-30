import React, { useState } from 'react';
import {
  Paper,
  Typography,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Button,
  SelectChangeEvent,
} from '@mui/material';
import { useCreateFriendship, useCreateFollow } from '../hooks/useApi';
import { User } from '../types/api';

interface SocialActionsProps {
  users: User[];
  selectedUserId: number | null;
  onActionComplete: () => void;
}

export const SocialActions: React.FC<SocialActionsProps> = ({
  users,
  selectedUserId,
  onActionComplete,
}) => {
  const [fromUserId, setFromUserId] = useState<string>('');
  const [toUserId, setToUserId] = useState<string>('');
  const [actionType, setActionType] = useState<'friendship' | 'follow'>('friendship');

  const createFriendshipMutation = useCreateFriendship();
  const createFollowMutation = useCreateFollow();

  const handleAction = async () => {
    if (!fromUserId || !toUserId) return;

    const fromId = parseInt(fromUserId);
    const toId = parseInt(toUserId);

    if (actionType === 'friendship') {
      await createFriendshipMutation.mutateAsync({
        from_user_id: fromId,
        to_user_id: toId,
      });
    } else {
      await createFollowMutation.mutateAsync({
        from_user_id: fromId,
        to_user_id: toId,
      });
    }
    onActionComplete();
  };

  return (
    <Paper sx={{ p: 2, height: '100%' }}>
      <Typography variant="h6" gutterBottom>
        Social Actions
      </Typography>
      <FormControl fullWidth sx={{ mb: 2 }}>
        <InputLabel>From User</InputLabel>
        <Select
          value={fromUserId}
          label="From User"
          onChange={(e: SelectChangeEvent) => setFromUserId(e.target.value)}
        >
          {users.map((user) => (
            <MenuItem key={user.id} value={user.id}>
              {user.full_name || user.username}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl fullWidth sx={{ mb: 2 }}>
        <InputLabel>To User</InputLabel>
        <Select
          value={toUserId}
          label="To User"
          onChange={(e: SelectChangeEvent) => setToUserId(e.target.value)}
        >
          {users.map((user) => (
            <MenuItem key={user.id} value={user.id}>
              {user.full_name || user.username}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl fullWidth sx={{ mb: 2 }}>
        <InputLabel>Action</InputLabel>
        <Select
          value={actionType}
          label="Action"
          onChange={(e: SelectChangeEvent) => setActionType(e.target.value as 'friendship' | 'follow')}
        >
          <MenuItem value="friendship">Friendship</MenuItem>
          <MenuItem value="follow">Follow</MenuItem>
        </Select>
      </FormControl>
      <Button
        variant="contained"
        onClick={handleAction}
        disabled={createFriendshipMutation.isPending || createFollowMutation.isPending}
      >
        Execute Action
      </Button>
    </Paper>
  );
};
