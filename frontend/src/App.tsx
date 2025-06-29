import React, { useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import {
  ThemeProvider,
  createTheme,
  CssBaseline,
  Box,
  AppBar,
  Toolbar,
  Typography,
  Container,
  Grid,
  Paper,
  Button,
  Chip,
  Alert,
  Snackbar,
} from '@mui/material';
import {
  Dashboard as DashboardIcon,
  Psychology as PsychologyIcon,
  Refresh as RefreshIcon,
} from '@mui/icons-material';

import { GraphVisualization } from './components/GraphVisualization';
import { UserManagement } from './components/UserManagement';
import { SocialActions } from './components/SocialActions';
import { useGraphData, useHealthCheck, useSeedSampleData, useUsers } from './hooks/useApi';

// Create theme
const theme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: '#667eea',
    },
    secondary: {
      main: '#764ba2',
    },
    background: {
      default: '#f5f5f5',
    },
  },
  typography: {
    h4: {
      fontWeight: 600,
    },
    h6: {
      fontWeight: 600,
    },
  },
  components: {
    MuiPaper: {
      styleOverrides: {
        root: {
          backgroundImage: 'none',
        },
      },
    },
  },
});

// Create query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

const AppContent: React.FC = () => {
  const [selectedUserId, setSelectedUserId] = useState<number | null>(null);
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const [snackbarMessage, setSnackbarMessage] = useState('');

  // API hooks
  const { data: healthStatus } = useHealthCheck();
  const {
    data: graphData,
    isLoading: graphLoading,
    error: graphError,
    refetch: refetchGraph
  } = useGraphData(); // Remove maxUsers and viewerId for now to fetch all data
  const { data: users = [], refetch: refetchUsers } = useUsers();
  const seedDataMutation = useSeedSampleData();

  const handleNodeClick = (nodeId: string) => {
    setSelectedUserId(parseInt(nodeId));
  };

  const handleSeedData = async () => {
    try {
      await seedDataMutation.mutateAsync();
      setSnackbarMessage('Sample data seeded successfully!');
      setSnackbarOpen(true);
    } catch (error) {
      setSnackbarMessage('Failed to seed sample data');
      setSnackbarOpen(true);
    }
  };

  const handleRefreshGraph = () => {
    refetchGraph();
    setSnackbarMessage('Graph data refreshed');
    setSnackbarOpen(true);
  };

  const handleActionComplete = () => {
    refetchGraph();
    refetchUsers();
  };

  return (
    <Box sx={{ flexGrow: 1, minHeight: '100vh' }}>
      {/* App Bar */}
      <AppBar position="static" elevation={0}>
        <Toolbar>
          <PsychologyIcon sx={{ mr: 2 }} />
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            TAO Database - Social Graph Explorer
          </Typography>

          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Chip
              icon={<DashboardIcon />}
              label={healthStatus || 'Connecting...'}
              color="success"
              variant="outlined"
              size="small"
            />

            <Button
              variant="outlined"
              color="inherit"
              startIcon={<RefreshIcon />}
              onClick={handleRefreshGraph}
              disabled={graphLoading}
              size="small"
            >
              Refresh
            </Button>

            <Button
              variant="contained"
              color="secondary"
              onClick={handleSeedData}
              disabled={seedDataMutation.isPending}
              size="small"
            >
              {seedDataMutation.isPending ? 'Seeding...' : 'Seed Data'}
            </Button>
          </Box>
        </Toolbar>
      </AppBar>

      {/* Main Content */}
      <Container maxWidth={false} sx={{ mt: 3, mb: 3 }}>
        <Grid container spacing={3} sx={{ height: 'calc(100vh - 180px)' }}>
          {/* Left Panel - User Management */}
          <Grid item xs={12} lg={3}>
            <UserManagement
              selectedUserId={selectedUserId}
              onUserSelect={setSelectedUserId}
            />
          </Grid>

          {/* Center Panel - Graph Visualization */}
          <Grid item xs={12} lg={6}>
            {graphData ? (
              <GraphVisualization
                graphData={graphData}
                onNodeClick={handleNodeClick}
                selectedNodeId={selectedUserId?.toString()}
                isLoading={graphLoading}
                error={graphError}
                onRefresh={handleRefreshGraph}
              />
            ) : (
              <Paper sx={{
                height: '100%',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexDirection: 'column',
                gap: 2,
              }}>
                <Typography variant="h6" color="text.secondary">
                  No graph data available
                </Typography>
                <Button
                  variant="contained"
                  onClick={handleSeedData}
                  disabled={seedDataMutation.isPending}
                >
                  {seedDataMutation.isPending ? 'Loading...' : 'Generate Sample Data'}
                </Button>
              </Paper>
            )}
          </Grid>

          {/* Right Panel - Social Actions */}
          <Grid item xs={12} lg={3}>
            <SocialActions
              users={users}
              selectedUserId={selectedUserId}
              onActionComplete={handleActionComplete}
            />
          </Grid>
        </Grid>
      </Container>

      {/* Footer */}
      <Box
        component="footer"
        sx={{
          position: 'fixed',
          bottom: 0,
          left: 0,
          right: 0,
          backgroundColor: 'background.paper',
          borderTop: 1,
          borderColor: 'divider',
          p: 2,
          zIndex: 1000,
        }}
      >
        <Container maxWidth={false}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <Typography variant="body2" color="text.secondary">
              🌐 TAO Database System - Inspired by Meta's TAO Architecture
            </Typography>

            <Box sx={{ display: 'flex', gap: 2 }}>
              <Chip
                size="small"
                label={`${graphData?.nodes?.length || 0} Users`}
                color="primary"
                variant="outlined"
              />
              <Chip
                size="small"
                label={`${graphData?.edges?.length || 0} Connections`}
                color="secondary"
                variant="outlined"
              />
            </Box>
          </Box>
        </Container>
      </Box>

      {/* Snackbar for notifications */}
      <Snackbar
        open={snackbarOpen}
        autoHideDuration={4000}
        onClose={() => setSnackbarOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert severity="info" onClose={() => setSnackbarOpen(false)}>
          {snackbarMessage}
        </Alert>
      </Snackbar>
    </Box>
  );
};

const App: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        <AppContent />
        {/* <ReactQueryDevtools initialIsOpen={false} /> */}
      </ThemeProvider>
    </QueryClientProvider>
  );
};

export default App;
