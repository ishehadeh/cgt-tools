//! Simple graph implementation

use std::collections::VecDeque;

pub mod directed;
pub mod undirected;

#[allow(missing_docs)]
pub trait Graph: Sized {
    type VertexIter: Iterator<Item = usize> + Clone;

    type AdjacentIter<'g>: Iterator<Item = usize>
    where
        Self: 'g;
    type DegreeIter<'g>: Iterator<Item = usize>
    where
        Self: 'g;
    type EdgesIter<'g>: Iterator<Item = (usize, usize)>
    where
        Self: 'g;

    fn empty(size: usize) -> Self;

    /// Get number of vertices in the graph.
    fn size(&self) -> usize;

    fn vertices(&self) -> Self::VertexIter;

    fn add_vertex(&mut self) -> usize;

    fn remove_vertex(&mut self, vertex_to_remove: usize);

    fn connect(&mut self, lhs_vertex: usize, rhs_vertex: usize, connect: bool);

    fn adjacent_to<'g>(&'g self, vertex: usize) -> Self::AdjacentIter<'g>;

    fn are_adjacent(&self, lhs_vertex: usize, rhs_vertex: usize) -> bool {
        self.adjacent_to(lhs_vertex)
            .any(|adjacent| adjacent == rhs_vertex)
    }

    fn edges<'g>(&'g self) -> Self::EdgesIter<'g>;

    fn degrees<'g>(&'g self) -> Self::DegreeIter<'g>;

    #[inline]
    fn from_flat_matrix(size: usize, matrix: &[bool]) -> Option<Self> {
        if matrix.len() != size * size {
            return None;
        }

        let mut g = Self::empty(size);
        for u in g.vertices() {
            for v in g.vertices() {
                g.connect(u, v, matrix[size * u + v]);
            }
        }

        Some(g)
    }

    #[inline]
    fn from_matrix(size: usize, matrix: &[&[bool]]) -> Option<Self> {
        let vec: Vec<bool> = matrix.iter().map(|r| r.iter()).flatten().copied().collect();
        Self::from_flat_matrix(size, &vec)
    }

    /// Get graph degree (highest vertex degree)
    #[inline]
    fn degree(&self) -> usize {
        self.degrees().max().unwrap_or(0)
    }

    /// Check if graph is connected
    #[inline]
    fn is_connected(&self) -> bool {
        if self.size() == 0 {
            return true;
        }

        let mut seen = vec![false; self.size()];
        let mut queue: VecDeque<usize> = VecDeque::with_capacity(self.size());

        seen[0] = true;
        queue.push_back(0);

        while let Some(v) = queue.pop_front() {
            for u in self.vertices() {
                if self.are_adjacent(v, u) && v != u && !seen[u] {
                    seen[u] = true;
                    queue.push_back(u);
                }
            }
        }

        seen.into_iter().all(|b| b)
    }

    /// Create a graph from list of edges
    #[inline]
    fn from_edges(size: usize, edges: &[(usize, usize)]) -> Self {
        let mut graph = Self::empty(size);
        for (v, u) in edges {
            graph.connect(*v, *u, true);
        }
        graph
    }
}
