import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import {
  Box,
  Paper,
  Typography,
  CircularProgress,
  Alert,
  Tooltip,
  IconButton,
  Chip,
} from '@mui/material';
import { Refresh as RefreshIcon, ZoomIn, ZoomOut, CenterFocusStrong } from '@mui/icons-material';
import { GraphData, D3Node, D3Edge } from '../types/api';

interface GraphVisualizationProps {
  graphData: GraphData;
  onNodeClick?: (nodeId: string) => void;
  selectedNodeId?: string;
  isLoading?: boolean;
  error?: any;
  onRefresh?: () => void;
}

export const GraphVisualization: React.FC<GraphVisualizationProps> = ({
  graphData,
  onNodeClick,
  selectedNodeId,
  isLoading,
  error,
  onRefresh,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const simulationRef = useRef<d3.Simulation<D3Node, D3Edge> | null>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });
  const [zoom, setZoom] = useState(1);

  // Handle resize
  useEffect(() => {
    const handleResize = () => {
      if (svgRef.current?.parentElement) {
        const rect = svgRef.current.parentElement.getBoundingClientRect();
        setDimensions({
          width: rect.width - 40, // Account for padding
          height: Math.max(500, rect.height - 100),
        });
      }
    };

    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  useEffect(() => {
    if (!graphData || !svgRef.current) return;

    const svg = d3.select(svgRef.current);
    const { width, height } = dimensions;

    // Clear previous content
    svg.selectAll('*').remove();

    // Create container group for zoom/pan
    const container = svg
      .append('g')
      .attr('class', 'graph-container');

    // Setup zoom behavior
    const zoomBehavior = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        container.attr('transform', event.transform);
        setZoom(event.transform.k);
      });

    svg.call(zoomBehavior);

    // Prepare data
    const nodes: D3Node[] = (graphData.nodes || []).map(d => ({ ...d }));
    const edges: D3Edge[] = (graphData.edges || []).map(edge => ({ ...edge }));

    console.log("GraphVisualization: Nodes received:", nodes);
    console.log("GraphVisualization: Edges received:", edges);

    // Create simulation
    const simulation = d3.forceSimulation<D3Node>(nodes)
      .force('link', d3.forceLink<D3Node, D3Edge>(edges).id(d => d.id).distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30));

    simulationRef.current = simulation;

    // Create links
    const link = container
      .append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(edges)
      .enter()
      .append('line')
      .attr('class', d => `link link-${d.edge_type}`)
      .attr('stroke-width', d => Math.sqrt(d.weight) * 2)
      .attr('stroke', d => {
        switch (d.edge_type) {
          case 'friendship': return '#4CAF50';
          case 'follows': return '#2196F3';
          case 'like': return '#E91E63';
          default: return '#999';
        }
      })
      .attr('stroke-dasharray', d => d.edge_type === 'follows' ? '5,5' : 'none')
      .attr('opacity', 0.7);

    // Create link labels
    const linkLabels = container
      .append('g')
      .attr('class', 'link-labels')
      .selectAll('text')
      .data(edges.filter(d => zoom > 1.5)) // Only show labels when zoomed in
      .enter()
      .append('text')
      .attr('class', 'link-label')
      .attr('text-anchor', 'middle')
      .attr('font-size', '10px')
      .attr('fill', '#666')
      .text(d => d.edge_type);

    // Create nodes
    const node = container
      .append('g')
      .attr('class', 'nodes')
      .selectAll('circle')
      .data(nodes)
      .enter()
      .append('circle')
      .attr('class', d => `node node-${d.node_type} ${d.verified ? 'verified' : ''} ${selectedNodeId === d.id ? 'selected' : ''}`)
      .attr('r', 15)
      .attr('fill', d => {
        if (selectedNodeId === d.id) return '#FF5722';
        return d.verified ? '#2196F3' : '#4CAF50';
      })
      .attr('stroke', d => d.verified ? '#FFD700' : '#fff')
      .attr('stroke-width', d => {
        if (selectedNodeId === d.id) return 4;
        return d.verified ? 3 : 2;
      })
      .style('cursor', 'pointer')
      .call(d3.drag<SVGCircleElement, D3Node>()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on('drag', (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        }))
      .on('click', (event, d) => {
        event.stopPropagation();
        onNodeClick?.(d.id);
      })
      .on('mouseover', function(event, d) {
        // Highlight connected edges
        link
          .style('opacity', l =>
            (l.source as D3Node).id === d.id || (l.target as D3Node).id === d.id ? 1 : 0.2
          );

        // Scale up the node
        d3.select(this)
          .transition()
          .duration(200)
          .attr('r', 20);
      })
      .on('mouseout', function() {
        // Reset edge opacity
        link.style('opacity', 0.7);

        // Reset node size
        d3.select(this)
          .transition()
          .duration(200)
          .attr('r', 15);
      });

    // Create node labels
    const labels = container
      .append('g')
      .attr('class', 'labels')
      .selectAll('text')
      .data(nodes)
      .enter()
      .append('text')
      .attr('class', 'node-label')
      .attr('text-anchor', 'middle')
      .attr('dy', 30)
      .attr('font-size', '12px')
      .attr('font-weight', 'bold')
      .attr('fill', '#333')
      .attr('stroke', '#fff')
      .attr('stroke-width', '3')
      .attr('paint-order', 'stroke')
      .text(d => d.name.split(' ')[0]) // First name only
      .style('pointer-events', 'none');

    // Update positions on tick
    simulation.on('tick', () => {
      link
        .attr('x1', d => (d.source as D3Node).x!)
        .attr('y1', d => (d.source as D3Node).y!)
        .attr('x2', d => (d.target as D3Node).x!)
        .attr('y2', d => (d.target as D3Node).y!);

      linkLabels
        .attr('x', d => ((d.source as D3Node).x! + (d.target as D3Node).x!) / 2)
        .attr('y', d => ((d.source as D3Node).y! + (d.target as D3Node).y!) / 2);

      node
        .attr('cx', d => d.x!)
        .attr('cy', d => d.y!);

      labels
        .attr('x', d => d.x!)
        .attr('y', d => d.y!);
    });

    // Add legend
    const legend = svg
      .append('g')
      .attr('class', 'legend')
      .attr('transform', `translate(20, 20)`);

    const legendData = [
      { color: '#4CAF50', text: 'Regular User', stroke: '#fff' },
      { color: '#2196F3', text: 'Verified User', stroke: '#FFD700' },
      { color: '#FF5722', text: 'Selected', stroke: '#fff' },
    ];

    const legendItems = legend
      .selectAll('.legend-item')
      .data(legendData)
      .enter()
      .append('g')
      .attr('class', 'legend-item')
      .attr('transform', (d, i) => `translate(0, ${i * 25})`);

    legendItems
      .append('circle')
      .attr('r', 8)
      .attr('fill', d => d.color)
      .attr('stroke', d => d.stroke)
      .attr('stroke-width', 2);

    legendItems
      .append('text')
      .attr('x', 20)
      .attr('y', 4)
      .attr('font-size', '12px')
      .attr('fill', '#333')
      .text(d => d.text);

    return () => {
      simulation.stop();
    };
  }, [graphData, dimensions, selectedNodeId, onNodeClick, zoom]);

  const handleZoomIn = () => {
    if (svgRef.current) {
      const svg = d3.select(svgRef.current);
      svg.transition().call(
        d3.zoom<SVGSVGElement, unknown>().scaleBy as any,
        1.5
      );
    }
  };

  const handleZoomOut = () => {
    if (svgRef.current) {
      const svg = d3.select(svgRef.current);
      svg.transition().call(
        d3.zoom<SVGSVGElement, unknown>().scaleBy as any,
        1 / 1.5
      );
    }
  };

  const handleCenter = () => {
    if (svgRef.current) {
      const svg = d3.select(svgRef.current);
      svg.transition().call(
        d3.zoom<SVGSVGElement, unknown>().transform as any,
        d3.zoomIdentity
      );
    }
  };

  if (error) {
    return (
      <Paper sx={{ p: 3, height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <Alert severity="error">
          Failed to load graph data: {error.message}
        </Alert>
      </Paper>
    );
  }

  return (
    <Paper sx={{ height: '100%', position: 'relative', overflow: 'hidden' }}>
      {/* Header */}
      <Box sx={{
        p: 2,
        borderBottom: 1,
        borderColor: 'divider',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between'
      }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="h6">
            Social Graph Visualization
          </Typography>
          <Chip
            size="small"
            label={`${graphData?.nodes?.length || 0} nodes, ${graphData?.edges?.length || 0} edges`}
            color="primary"
            variant="outlined"
          />
          <Chip
            size="small"
            label={`Zoom: ${Math.round(zoom * 100)}%`}
            color="secondary"
            variant="outlined"
          />
        </Box>

        <Box sx={{ display: 'flex', gap: 1 }}>
          <Tooltip title="Zoom In">
            <IconButton size="small" onClick={handleZoomIn}>
              <ZoomIn />
            </IconButton>
          </Tooltip>
          <Tooltip title="Zoom Out">
            <IconButton size="small" onClick={handleZoomOut}>
              <ZoomOut />
            </IconButton>
          </Tooltip>
          <Tooltip title="Center">
            <IconButton size="small" onClick={handleCenter}>
              <CenterFocusStrong />
            </IconButton>
          </Tooltip>
          <Tooltip title="Refresh">
            <IconButton size="small" onClick={onRefresh} disabled={isLoading}>
              <RefreshIcon />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Graph Container */}
      <Box sx={{ position: 'relative', height: 'calc(100% - 80px)' }}>
        {isLoading && (
          <Box sx={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'rgba(255, 255, 255, 0.8)',
            zIndex: 10,
          }}>
            <CircularProgress />
          </Box>
        )}

        <svg
          ref={svgRef}
          width="100%"
          height="100%"
          style={{ backgroundColor: '#fafafa' }}
        />
      </Box>

      {/* Instructions */}
      <Box sx={{
        position: 'absolute',
        bottom: 16,
        left: 16,
        backgroundColor: 'rgba(255, 255, 255, 0.9)',
        p: 1,
        borderRadius: 1,
        fontSize: '0.75rem',
        color: 'text.secondary'
      }}>
        <Typography variant="caption" display="block">
          • Click nodes to select • Drag to reposition • Mouse wheel to zoom
        </Typography>
      </Box>
    </Paper>
  );
};
