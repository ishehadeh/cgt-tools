//! Snort is played on undirected graph. In each turn Left colors a vertex blue and Right colors
//! a vertex red. Players can only choose a vertex that is adjecent to only empty vertices or to
//! vertices in their own color.

use crate::{
    graph::undirected::Graph,
    short::partizan::short_canonical_game::{Game, Moves, PartizanShortGame},
    short::partizan::transposition_table::TranspositionTable,
};
use num_derive::FromPrimitive;
use std::{collections::VecDeque, fmt::Write};

/// Color of Snort vertex. Note that we are taking tinting apporach rather than direct tracking
/// of adjacent colors.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, FromPrimitive)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(u8)]
pub enum VertexColor {
    /// Vertex without color, not connected to any taken vertices
    Empty = 0,

    /// Vertex that is adjecent to left
    TintLeft = 1,

    /// Vertex that is adjecent to right
    TintRight = 2,

    /// Vertex that is either taken or connected to both colors
    Taken = 3,
}

/// Position of a [snort](self) game
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    /// Vertices colors of the game graph
    pub vertices: Vec<VertexColor>,

    /// Get graph of the game. This includes only edges
    pub graph: Graph,
}

impl Position {
    /// Create new Snort position with all vertices empty.
    pub fn new(graph: Graph) -> Self {
        Self {
            vertices: vec![VertexColor::Empty; graph.size()],
            graph,
        }
    }

    // TODO: Perform that check
    /// Create a Snort position with initial colors. It's up to the user to ensure that no conflicting
    /// colors are connected in the graph.
    /// Returns `None` if `vertices` and `graph` have conflicting sizes.
    pub fn with_colors(vertices: Vec<VertexColor>, graph: Graph) -> Option<Self> {
        if vertices.len() != graph.size() {
            return None;
        }

        Some(Self { vertices, graph })
    }

    /// Get moves for a given player. Works only for `TintLeft` and `TintRight`.
    /// Any other input is undefined.
    fn moves_for<const COLOR: u8>(&self) -> Vec<Self> {
        // const ADT generics are unsable, so here we go
        let own_tint_color: VertexColor = num::FromPrimitive::from_u8(COLOR).unwrap();

        let mut moves = Vec::with_capacity(self.graph.size());

        // Vertices where player can move
        let move_vertices = self
            .vertices
            .iter()
            .enumerate()
            .filter(|(_, vertex_color)| {
                **vertex_color == own_tint_color || **vertex_color == VertexColor::Empty
            })
            .map(|(idx, _)| idx);

        // Go through list of vertices with legal move
        for move_vertex in move_vertices {
            let mut position: Position = self.clone();

            // Take vertex
            position.vertices[move_vertex] = VertexColor::Taken;

            // Disconnect `move_vertex` from adjecent vertices and tint them
            for adjacent_vertex in self.graph.adjacent_to(move_vertex) {
                // Disconnect move vertex from adjecent
                position.graph.connect(move_vertex, adjacent_vertex, false);

                // No loops in snort graphs
                if adjacent_vertex != move_vertex {
                    let adjacent_vertex_color = &mut position.vertices[adjacent_vertex];
                    // Tint adjacent vertex

                    if *adjacent_vertex_color == own_tint_color
                        || *adjacent_vertex_color == VertexColor::Empty
                    {
                        // If adjacent vertex is empty or tinted in own color, tint it in own
                        *adjacent_vertex_color = own_tint_color;
                    } else {
                        // Otherwise the vertex is tinted in opponents color, so no one can longer
                        // move there, thus we mark is as taken and disconnect from the graph
                        *adjacent_vertex_color = VertexColor::Taken;
                        for v in position.graph.vertices() {
                            position.graph.connect(v, adjacent_vertex, false);
                        }
                    }
                }
            }
            moves.push(position);
        }
        moves
    }

    /// BFS search to get the decompisitons, should be used only as a helper for [Self::decompositions]
    fn bfs(&self, visited: &mut Vec<bool>, v: usize) -> Self {
        let mut vertices_to_take: Vec<usize> = Vec::new();

        let mut q: VecDeque<usize> = VecDeque::new();
        q.push_back(v);
        visited[v] = true;

        while let Some(v) = q.pop_front() {
            vertices_to_take.push(v);

            for u in self.graph.adjacent_to(v) {
                if !visited[u] {
                    visited[u] = true;
                    q.push_back(u);
                }
            }
        }

        let mut new_graph = Graph::empty(vertices_to_take.len());
        for (new_v, old_v) in vertices_to_take.iter().enumerate() {
            for old_u in self.graph.adjacent_to(*old_v) {
                if let Some(new_u) = vertices_to_take.iter().position(|x| *x == old_u) {
                    new_graph.connect(new_v, new_u, true);
                }
            }
        }

        let mut new_vertices = Vec::with_capacity(vertices_to_take.len());
        for v in &vertices_to_take {
            new_vertices.push(self.vertices[*v]);
        }

        Self {
            vertices: new_vertices,
            graph: new_graph,
        }
    }

    /// Decompose the game graph into disconnected components
    ///
    /// # Examples
    ///
    /// ```
    /// use cgt::short::partizan::games::snort::Position;
    /// use cgt::graph::undirected::Graph;
    ///
    /// assert_eq!(
    ///     Position::new(Graph::from_edges(5, &[(0, 1), (0, 2), (1, 2), (3, 4)])).decompositions(),
    ///     vec![
    ///         Position::new(Graph::from_edges(3, &[(0, 1), (0, 2), (1, 2)])),
    ///         Position::new(Graph::from_edges(2, &[(0, 1)]))
    ///     ]
    /// );
    /// ```
    pub fn decompositions(&self) -> Vec<Self> {
        let mut visited = vec![false; self.vertices.len()];
        let mut res = Vec::new();

        for v in self.graph.vertices() {
            if !matches!(self.vertices[v], VertexColor::Taken) && !visited[v] {
                res.push(self.bfs(&mut visited, v));
            }
        }

        res
    }
}

#[test]
fn decomposition_works() {
    assert_eq!(
        Position::new(Graph::from_edges(3, &[(0, 1), (0, 2), (1, 2)])).decompositions(),
        vec![Position::new(Graph::from_edges(
            3,
            &[(0, 1), (0, 2), (1, 2)]
        ))]
    );
}

impl PartizanShortGame for Position {
    fn left_moves(&self) -> Vec<Self> {
        self.moves_for::<{ VertexColor::TintLeft as u8 }>()
    }

    fn right_moves(&self) -> Vec<Self> {
        self.moves_for::<{ VertexColor::TintRight as u8 }>()
    }
}

impl Position {
    /// Get the canonical form of the position.
    ///
    /// # Arguments
    ///
    /// `cache` - Shared cache of short combinatorial games.
    ///
    /// # Examples
    ///
    /// ```
    /// use cgt::graph::undirected::Graph;
    /// use cgt::short::partizan::short_canonical_game::PartizanShortGame;
    /// use cgt::short::partizan::games::snort::{Position, VertexColor};
    /// use cgt::short::partizan::transposition_table::TranspositionTable;
    ///
    /// let mut graph = Graph::empty(3);
    /// graph.connect(1, 2, true);
    ///
    /// let colors = vec![
    ///     VertexColor::TintLeft,
    ///     VertexColor::TintRight,
    ///     VertexColor::TintLeft,
    /// ];
    ///
    /// let position = Position::with_colors(colors, graph).unwrap();
    /// assert_eq!(position.left_moves().len(), 2);
    /// assert_eq!(position.right_moves().len(), 1);
    ///
    /// let cache = TranspositionTable::new();
    /// let game = position.canonical_form(&cache);
    /// assert_eq!(&cache.game_backend().print_game_to_str(&game), "1*");
    /// ```
    pub fn canonical_form(&self, cache: &TranspositionTable<Self>) -> Game {
        // TODO: move to trait

        if let Some(id) = cache.grids_get(self) {
            return id;
        }

        let mut result = cache.game_backend().construct_integer(0);
        for position in self.decompositions() {
            let sub_result = match cache.grids_get(&position) {
                Some(canonical_form) => canonical_form,
                None => {
                    let moves = Moves {
                        left: position
                            .left_moves()
                            .iter()
                            .map(|o| o.canonical_form(cache))
                            .collect(),
                        right: position
                            .right_moves()
                            .iter()
                            .map(|o| o.canonical_form(cache))
                            .collect(),
                    };

                    let canonical_form = cache.game_backend().construct_from_moves(moves);
                    cache.grids_insert(position, canonical_form);
                    canonical_form
                }
            };

            result = cache.game_backend().construct_sum(sub_result, result);
        }

        cache.grids_insert(self.clone(), result);
        result
    }
}

#[test]
fn no_moves() {
    let position = Position::new(Graph::empty(0));
    assert_eq!(position.left_moves(), vec![]);
    assert_eq!(position.right_moves(), vec![]);
}

impl Position {
    /// Render to a [graphviz](https://graphviz.org/) format, that can be later rendered to an
    /// image with external engine.
    pub fn to_graphviz(&self) -> String {
        let mut buf = String::new();

        write!(buf, "graph G {{").unwrap();

        for (v, color) in self.vertices.iter().enumerate() {
            let col = match color {
                VertexColor::Empty => "white",
                VertexColor::TintLeft => "blue",
                VertexColor::TintRight => "red",
                VertexColor::Taken => continue,
            };
            write!(buf, "{} [fillcolor={}, style=filled, shape=circle, fixedsize=true, width=1, height=1, fontsize=24];", v, col).unwrap();
        }

        for v in self.graph.vertices() {
            for u in self.graph.vertices() {
                if v < u && self.graph.are_adjacent(v, u) {
                    write!(buf, "{v} -- {u};").unwrap();
                }
            }
        }

        write!(buf, "}}").unwrap();
        buf
    }
}
