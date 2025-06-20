import React, { useState } from 'react';
import {
  Box,
  Paper,
  Typography,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  TextField,
  Grid,
  Alert,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Snackbar,
} from '@mui/material';
import {
  PersonAdd as PersonAddIcon,
  Favorite as FavoriteIcon,
  PersonAddAlt1 as FollowIcon,
  ExpandMore as ExpandMoreIcon,
  Group as GroupIcon,
  Article as ArticleIcon,
} from '@mui/icons-material';
import { TaoApiService } from '../services/api';
import { User, CreateFriendshipRequest, CreateFollowRequest, CreateLikeRequest, CreatePostRequest } from '../types/api';

interface SocialActionsProps {
  users: User[];
  selectedUserId?: number | null;
  onActionComplete?: () => void;
}

export const SocialActions: React.FC<SocialActionsProps> = ({
  users,
  selectedUserId,
  onActionComplete,
}) => {
  const [friendshipDialog, setFriendshipDialog] = useState(false);
  const [followDialog, setFollowDialog] = useState(false);
  const [likeDialog, setLikeDialog] = useState(false);
  const [postDialog, setPostDialog] = useState(false);
  const [snackbar, setSnackbar] = useState<{ open: boolean; message: string; severity: 'success' | 'error' }>({
    open: false,
    message: '',
    severity: 'success',
  });

  // Form states
  const [friendshipForm, setFriendshipForm] = useState<CreateFriendshipRequest>({
    user1_id: 0,
    user2_id: 0,
    relationship_type: 'friend',
  });

  const [followForm, setFollowForm] = useState<CreateFollowRequest>({
    follower_id: 0,
    followee_id: 0,
    follow_type: 'default',
  });

  const [likeForm, setLikeForm] = useState<CreateLikeRequest>({
    user_id: 0,
    target_id: 0,
    reaction_type: 'like',
  });

  const [postForm, setPostForm] = useState<CreatePostRequest>({
    author_id: 0,
    content: '',
    post_type: 'text',
    visibility: 'public',
  });

  const showSnackbar = (message: string, severity: 'success' | 'error' = 'success') => {
    setSnackbar({ open: true, message, severity });
  };

  const handleCreateFriendship = async () => {
    try {
      await TaoApiService.createFriendship(friendshipForm);
      setFriendshipDialog(false);
      setFriendshipForm({ user1_id: 0, user2_id: 0, relationship_type: 'friend' });
      showSnackbar('Friendship created successfully!');
      onActionComplete?.();
    } catch (error) {
      showSnackbar('Failed to create friendship', 'error');
    }
  };

  const handleCreateFollow = async () => {
    try {
      await TaoApiService.createFollow(followForm);
      setFollowDialog(false);
      setFollowForm({ follower_id: 0, followee_id: 0, follow_type: 'default' });
      showSnackbar('Follow relationship created successfully!');
      onActionComplete?.();
    } catch (error) {
      showSnackbar('Failed to create follow relationship', 'error');
    }
  };

  const handleCreateLike = async () => {
    try {
      await TaoApiService.createLike(likeForm);
      setLikeDialog(false);
      setLikeForm({ user_id: 0, target_id: 0, reaction_type: 'like' });
      showSnackbar('Like created successfully!');
      onActionComplete?.();
    } catch (error) {
      showSnackbar('Failed to create like', 'error');
    }
  };

  const handleCreatePost = async () => {
    try {
      await TaoApiService.createPost(postForm);
      setPostDialog(false);
      setPostForm({ author_id: 0, content: '', post_type: 'text', visibility: 'public' });
      showSnackbar('Post created successfully!');
      onActionComplete?.();
    } catch (error) {
      showSnackbar('Failed to create post', 'error');
    }
  };

  const handleSeedData = async () => {
    try {
      await TaoApiService.seedSampleData();
      showSnackbar('Sample data seeded successfully!');
      onActionComplete?.();
    } catch (error) {
      showSnackbar('Failed to seed data', 'error');
    }
  };

  const UserSelector: React.FC<{
    label: string;
    value: number;
    onChange: (value: number) => void;
    exclude?: number;
  }> = ({ label, value, onChange, exclude }) => (
    <FormControl fullWidth>
      <InputLabel>{label}</InputLabel>
      <Select
        value={value}
        label={label}
        onChange={(e) => onChange(e.target.value as number)}
      >
        {users
          .filter(user => exclude === undefined || user.id !== exclude)
          .map((user) => (
            <MenuItem key={user.id} value={user.id}>
              {user.full_name || user.username} (@{user.username})
            </MenuItem>
          ))}
      </Select>
    </FormControl>
  );

  return (
    <Paper sx={{ p: 2 }}>
      <Typography variant="h6" sx={{ mb: 2 }}>
        Social Actions
      </Typography>

      {selectedUserId && (
        <Alert severity="info" sx={{ mb: 2 }}>
          Selected User ID: {selectedUserId} - {users.find(u => u.id === selectedUserId)?.username}
        </Alert>
      )}

      <Accordion>
        <AccordionSummary expandIcon={<ExpandMoreIcon />}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <GroupIcon />
            <Typography>Relationships</Typography>
          </Box>
        </AccordionSummary>
        <AccordionDetails>
          <Grid container spacing={2}>
            <Grid item xs={12} sm={6}>
              <Button
                fullWidth
                variant="outlined"
                startIcon={<PersonAddIcon />}
                onClick={() => setFriendshipDialog(true)}
                disabled={users.length < 2}
              >
                Create Friendship
              </Button>
            </Grid>
            <Grid item xs={12} sm={6}>
              <Button
                fullWidth
                variant="outlined"
                startIcon={<FollowIcon />}
                onClick={() => setFollowDialog(true)}
                disabled={users.length < 2}
              >
                Create Follow
              </Button>
            </Grid>
          </Grid>
        </AccordionDetails>
      </Accordion>

      <Accordion>
        <AccordionSummary expandIcon={<ExpandMoreIcon />}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <ArticleIcon />
            <Typography>Content</Typography>
          </Box>
        </AccordionSummary>
        <AccordionDetails>
          <Grid container spacing={2}>
            <Grid item xs={12} sm={6}>
              <Button
                fullWidth
                variant="outlined"
                startIcon={<ArticleIcon />}
                onClick={() => setPostDialog(true)}
                disabled={users.length === 0}
              >
                Create Post
              </Button>
            </Grid>
            <Grid item xs={12} sm={6}>
              <Button
                fullWidth
                variant="outlined"
                startIcon={<FavoriteIcon />}
                onClick={() => setLikeDialog(true)}
                disabled={users.length < 2}
              >
                Create Like
              </Button>
            </Grid>
          </Grid>
        </AccordionDetails>
      </Accordion>

      <Box sx={{ mt: 2 }}>
        <Button
          fullWidth
          variant="contained"
          color="secondary"
          onClick={handleSeedData}
        >
          Seed Sample Data
        </Button>
      </Box>

      {/* Friendship Dialog */}
      <Dialog open={friendshipDialog} onClose={() => setFriendshipDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Friendship</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
            <UserSelector
              label="User 1"
              value={friendshipForm.user1_id}
              onChange={(value) => setFriendshipForm({ ...friendshipForm, user1_id: value })}
            />
            <UserSelector
              label="User 2"
              value={friendshipForm.user2_id}
              onChange={(value) => setFriendshipForm({ ...friendshipForm, user2_id: value })}
              exclude={friendshipForm.user1_id}
            />
            <FormControl fullWidth>
              <InputLabel>Relationship Type</InputLabel>
              <Select
                value={friendshipForm.relationship_type}
                label="Relationship Type"
                onChange={(e) => setFriendshipForm({ ...friendshipForm, relationship_type: e.target.value })}
              >
                <MenuItem value="friend">Friend</MenuItem>
                <MenuItem value="family">Family</MenuItem>
                <MenuItem value="colleague">Colleague</MenuItem>
              </Select>
            </FormControl>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setFriendshipDialog(false)}>Cancel</Button>
          <Button
            onClick={handleCreateFriendship}
            variant="contained"
            disabled={!friendshipForm.user1_id || !friendshipForm.user2_id || friendshipForm.user1_id === friendshipForm.user2_id}
          >
            Create Friendship
          </Button>
        </DialogActions>
      </Dialog>

      {/* Follow Dialog */}
      <Dialog open={followDialog} onClose={() => setFollowDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Follow Relationship</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
            <UserSelector
              label="Follower"
              value={followForm.follower_id}
              onChange={(value) => setFollowForm({ ...followForm, follower_id: value })}
            />
            <UserSelector
              label="Followee"
              value={followForm.followee_id}
              onChange={(value) => setFollowForm({ ...followForm, followee_id: value })}
              exclude={followForm.follower_id}
            />
            <FormControl fullWidth>
              <InputLabel>Follow Type</InputLabel>
              <Select
                value={followForm.follow_type}
                label="Follow Type"
                onChange={(e) => setFollowForm({ ...followForm, follow_type: e.target.value })}
              >
                <MenuItem value="default">Default</MenuItem>
                <MenuItem value="close_friend">Close Friend</MenuItem>
                <MenuItem value="acquaintance">Acquaintance</MenuItem>
              </Select>
            </FormControl>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setFollowDialog(false)}>Cancel</Button>
          <Button
            onClick={handleCreateFollow}
            variant="contained"
            disabled={!followForm.follower_id || !followForm.followee_id || followForm.follower_id === followForm.followee_id}
          >
            Create Follow
          </Button>
        </DialogActions>
      </Dialog>

      {/* Post Dialog */}
      <Dialog open={postDialog} onClose={() => setPostDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Post</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
            <UserSelector
              label="Author"
              value={postForm.author_id}
              onChange={(value) => setPostForm({ ...postForm, author_id: value })}
            />
            <TextField
              label="Content"
              value={postForm.content}
              onChange={(e) => setPostForm({ ...postForm, content: e.target.value })}
              multiline
              rows={4}
              fullWidth
              required
            />
            <FormControl fullWidth>
              <InputLabel>Post Type</InputLabel>
              <Select
                value={postForm.post_type}
                label="Post Type"
                onChange={(e) => setPostForm({ ...postForm, post_type: e.target.value })}
              >
                <MenuItem value="text">Text</MenuItem>
                <MenuItem value="photo">Photo</MenuItem>
                <MenuItem value="video">Video</MenuItem>
                <MenuItem value="link">Link</MenuItem>
              </Select>
            </FormControl>
            <FormControl fullWidth>
              <InputLabel>Visibility</InputLabel>
              <Select
                value={postForm.visibility}
                label="Visibility"
                onChange={(e) => setPostForm({ ...postForm, visibility: e.target.value })}
              >
                <MenuItem value="public">Public</MenuItem>
                <MenuItem value="friends">Friends</MenuItem>
                <MenuItem value="private">Private</MenuItem>
              </Select>
            </FormControl>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setPostDialog(false)}>Cancel</Button>
          <Button
            onClick={handleCreatePost}
            variant="contained"
            disabled={!postForm.author_id || !postForm.content.trim()}
          >
            Create Post
          </Button>
        </DialogActions>
      </Dialog>

      {/* Like Dialog */}
      <Dialog open={likeDialog} onClose={() => setLikeDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Like</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
            <UserSelector
              label="User"
              value={likeForm.user_id}
              onChange={(value) => setLikeForm({ ...likeForm, user_id: value })}
            />
            <TextField
              label="Target ID (Post/Comment ID)"
              type="number"
              value={likeForm.target_id}
              onChange={(e) => setLikeForm({ ...likeForm, target_id: parseInt(e.target.value) || 0 })}
              fullWidth
              helperText="Enter the ID of the post or comment to like"
            />
            <FormControl fullWidth>
              <InputLabel>Reaction Type</InputLabel>
              <Select
                value={likeForm.reaction_type}
                label="Reaction Type"
                onChange={(e) => setLikeForm({ ...likeForm, reaction_type: e.target.value })}
              >
                <MenuItem value="like">👍 Like</MenuItem>
                <MenuItem value="love">❤️ Love</MenuItem>
                <MenuItem value="laugh">😂 Laugh</MenuItem>
                <MenuItem value="wow">😮 Wow</MenuItem>
                <MenuItem value="sad">😢 Sad</MenuItem>
                <MenuItem value="angry">😡 Angry</MenuItem>
              </Select>
            </FormControl>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setLikeDialog(false)}>Cancel</Button>
          <Button
            onClick={handleCreateLike}
            variant="contained"
            disabled={!likeForm.user_id || !likeForm.target_id}
          >
            Create Like
          </Button>
        </DialogActions>
      </Dialog>

      {/* Snackbar for notifications */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={4000}
        onClose={() => setSnackbar({ ...snackbar, open: false })}
      >
        <Alert severity={snackbar.severity} onClose={() => setSnackbar({ ...snackbar, open: false })}>
          {snackbar.message}
        </Alert>
      </Snackbar>
    </Paper>
  );
};