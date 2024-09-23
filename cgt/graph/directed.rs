//! Directed graph

use core::ops::Range;
use std::{fmt::Display, iter::FusedIterator};

/// Directed graph
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirectedGraph {
    size: usize,
    adjacency_matrix: Vec<bool>,
}

impl Display for DirectedGraph {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, elem) in self.adjacency_matrix.iter().enumerate() {
            write!(f, "{}", u8::from(*elem))?;
            if (idx + 1) % self.size == 0 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl DirectedGraph {
    /// Create an empty graph without any edges between vertices
    #[inline]
    pub fn empty(size: usize) -> Self {
        Self {
            size,
            adjacency_matrix: vec![false; size * size],
        }
    }

    /// Create a graph from flattened adjecency matrix. Must be correct length
    #[inline]
    pub fn from_vec(size: usize, vec: Vec<bool>) -> Option<Self> {
        if vec.len() != size * size {
            return None;
        }

        Some(Self {
            size,
            adjacency_matrix: vec,
        })
    }

    /// Create a graph from adjecency matrix. Must be correct length
    #[inline]
    pub fn from_matrix(size: usize, matrix: &[Vec<bool>]) -> Option<Self> {
        let vec: Vec<bool> = matrix.iter().flatten().copied().collect();
        Self::from_vec(size, vec)
    }

    /// Get number of vertices in the graph.
    #[inline]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// Check if two vertices are adjacent.
    #[inline]
    pub fn are_adjacent(&self, out_vertex: usize, in_vertex: usize) -> bool {
        self.adjacency_matrix[self.size * in_vertex + out_vertex]
    }

    /// Connect two vertices with an edge.
    #[inline]
    pub fn connect(&mut self, out_vertex: usize, in_vertex: usize, connect: bool) {
        self.adjacency_matrix[self.size * in_vertex + out_vertex] = connect;
    }

    /// Get vertices adjacent to `out_vertex`.
    #[inline]
    pub fn adjacent_to(&self, out_vertex: usize) -> AdjacentIter {
        AdjacentIter {
            vertex: out_vertex,
            idx: 0,
            graph: self,
        }
    }

    /// Get edges of the graph
    #[inline]
    pub fn edges(&self) -> EdgesIter {
        EdgesIter {
            u: 0,
            v: 0,
            graph: self,
        }
    }

    /// Get iterator over vertices
    #[inline]
    pub const fn vertices(&self) -> Range<usize> {
        0..self.size()
    }

    /// Add a new disconnected vertex at the end of the graph
    #[inline]
    pub fn add_vertex(&mut self) {
        let mut new_graph = Self::empty(self.size() + 1);
        for in_v in self.vertices() {
            for out_v in self.vertices() {
                new_graph.connect(out_v, in_v, self.are_adjacent(out_v, in_v));
            }
        }
        *self = new_graph;
    }

    /// Remove a given vertex from the graph, remove all its edges
    #[inline]
    pub fn remove_vertex(&mut self, vertex_to_remove: usize) {
        debug_assert!(self.size() > 0, "Graph has no vertices");
        let mut new_graph = Self::empty(self.size() - 1);

        for in_v in new_graph.vertices() {
            for out_v in new_graph.vertices() {
                new_graph.connect(
                    out_v,
                    in_v,
                    self.are_adjacent(
                        // Skip over vertex we're removing
                        out_v + (out_v >= vertex_to_remove) as usize,
                        in_v + (in_v >= vertex_to_remove) as usize,
                    ),
                );
            }
        }

        *self = new_graph;
    }
}

/// Iterator over graph edges, constructed with [`Graph::edges`].
pub struct EdgesIter<'graph> {
    u: usize,
    v: usize,
    graph: &'graph DirectedGraph,
}

impl<'graph> Iterator for EdgesIter<'graph> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.u >= self.graph.size() {
                self.u = 0;
                self.v += 1;
            }

            if self.v >= self.graph.size() {
                return None;
            }

            if self.graph.are_adjacent(self.u, self.v) {
                let res = Some((self.u, self.v));
                self.u += 1;
                return res;
            }
            self.u += 1;
        }
    }
}

impl<'graph> FusedIterator for EdgesIter<'graph> {}

/// Iterator of adjacent vertices. Obtained by calling [`Graph::adjacent_to`]
#[derive(Debug)]
pub struct AdjacentIter<'graph> {
    vertex: usize,
    idx: usize,
    graph: &'graph DirectedGraph,
}

impl<'graph> Iterator for AdjacentIter<'graph> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.idx >= self.graph.size {
                return None;
            }
            if self.graph.are_adjacent(self.vertex, self.idx) {
                let res = Some(self.idx);
                self.idx += 1;
                return res;
            }
            self.idx += 1;
        }
    }
}

impl<'graph> FusedIterator for AdjacentIter<'graph> {}

#[test]
fn adds_new_vertex() {
    let mut g = test_matrix();
    assert_eq!(
        &format!("{g}"),
        "0101\n\
	 0000\n\
	 0001\n\
	 0100\n"
    );

    // adds one empty row and column to previous matrix
    g.add_vertex();
    assert_eq!(
        &format!("{g}"),
        "01010\n\
	 00000\n\
	 00010\n\
	 01000\n\
         00000\n"
    );

    g.remove_vertex(1);
    assert_eq!(
        &format!("{g}"),
        "0010\n\
	 0010\n\
	 0000\n\
         0000\n"
    );
}

/// ```text
/// 1 -> 3 -> 2
///  \   |
///   \  v
///    > 0
/// ```
#[cfg(test)]
fn test_matrix() -> DirectedGraph {
    let mut m = DirectedGraph::empty(4);
    m.connect(3, 0, true);
    m.connect(3, 2, true);
    m.connect(1, 3, true);
    m.connect(1, 0, true);
    m
}

#[test]
fn set_adjacency_matrix() {
    let m = test_matrix();
    assert_eq!(
        m,
        DirectedGraph::from_vec(
            4,
            vec![
                false, true, false, true, false, false, false, false, false, false, false, true,
                false, true, false, false
            ]
        )
        .unwrap()
    );
}

#[test]
fn test_adjacency() {
    let m = test_matrix();
    assert_eq!(m.adjacent_to(0).collect::<Vec<_>>(), vec![]);
    assert_eq!(m.adjacent_to(1).collect::<Vec<_>>(), vec![0, 3]);
    assert_eq!(m.adjacent_to(2).collect::<Vec<_>>(), vec![]);
    assert_eq!(m.adjacent_to(3).collect::<Vec<_>>(), vec![0, 2]);
}

#[test]
fn test_edges() {
    let m = test_matrix();
    assert_eq!(
        m.edges().collect::<Vec<_>>(),
        vec![(1, 0), (3, 0), (3, 2), (1, 3)]
    );
}
