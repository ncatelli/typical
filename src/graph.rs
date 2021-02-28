use std::fmt::Debug;

type NodeId = crate::types::EntityId<usize>;

pub trait Edge: Copy {
    fn new(a: NodeId, b: NodeId) -> Self;
    /// Return the source node where (A, B) -> A
    fn source(&self) -> NodeId;
    /// Return the destination node where (A, B) -> B
    fn destination(&self) -> NodeId;
    /// Return the inverse edge where (A, B) -> (B, A)
    fn inverse(&self) -> Self;
}

impl Edge for (NodeId, NodeId) {
    fn new(a: NodeId, b: NodeId) -> Self {
        (a, b)
    }

    fn source(&self) -> NodeId {
        self.0
    }

    fn destination(&self) -> NodeId {
        self.1
    }

    fn inverse(&self) -> Self {
        (self.1, self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Graph<N, E>
where
    N: crate::types::AbstractEntity,
    E: Edge,
{
    /// Stores all nodes indexed by
    nodes: Vec<N>,
    edges: Vec<Vec<E>>,
}

impl<N, E> Graph<N, E>
where
    N: crate::types::AbstractEntity,
    E: Edge,
{
    pub fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn mut_add_node(&mut self, new_node: N) -> NodeId {
        self.nodes.push(new_node);
        self.edges.push(vec![]);

        NodeId::new(self.nodes.len() - 1)
    }

    pub fn mut_add_directed_edge(&mut self, a: NodeId, b: NodeId) -> E {
        let a_to_b = E::new(a, b);
        self.edges[usize::from(b)].push(a_to_b);
        self.edges[usize::from(a)].push(a_to_b);

        a_to_b
    }

    pub fn mut_add_undirected_edge(&mut self, a: NodeId, b: NodeId) -> E {
        let edge = E::new(a, b);
        let inverse_edge = edge.inverse();
        self.edges[usize::from(a)].push(edge);
        self.edges[usize::from(a)].push(inverse_edge);
        self.edges[usize::from(b)].push(edge);
        self.edges[usize::from(b)].push(inverse_edge);

        edge
    }

    pub fn upper_bounds(&self, eid: NodeId) -> Option<Vec<NodeId>> {
        let idx = usize::from(eid);
        let edges: Vec<NodeId> = self.edges[idx]
            .iter()
            .map(|&e| {
                if e.source() == eid {
                    Some(e.destination())
                } else {
                    None
                }
            })
            .filter_map(|x| x)
            .collect();

        if edges.is_empty() {
            None
        } else {
            Some(edges)
        }
    }
}

// Immutable modifications
impl<N, E> Graph<N, E>
where
    N: crate::types::AbstractEntity,
    E: Edge,
{
    pub fn add_node(mut self, new_node: N) -> (NodeId, Self) {
        let new_key = self.mut_add_node(new_node);
        (new_key, self)
    }

    pub fn add_directed_edge(mut self, a: NodeId, b: NodeId) -> (E, Self) {
        let new_edge = self.mut_add_directed_edge(a, b);
        (new_edge, self)
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::*;
    use crate::types::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum AbstractTypes {
        Any,
        Numeric,
        Integer,
        Unsigned(usize),
        Signed(usize),
        String,
    }

    impl AbstractEntity for AbstractTypes {
        fn unconstrained_type() -> Self {
            Self::Any
        }

        fn arity(&self) -> Option<usize> {
            match self {
                Self::Any | Self::Numeric | Self::Integer => None,
                _ => Some(0),
            }
        }

        fn converge(&self, other: &Self) -> Result<Self, TypeError> {
            match (self, other) {
                (Self::Any, o) | (o, Self::Any) => Ok(*o),
                (Self::String, Self::String) => Ok(Self::String),
                (Self::String, _) | (_, Self::String) => Err(TypeError::Converge),
                (Self::Numeric, o) | (o, Self::Numeric) => Ok(*o), // o can only be Numeric, Integer, Unsigned, Signed, or U8.
                (Self::Integer, o) | (o, Self::Integer) => Ok(*o), // o can only be  Integer, Unsigned, Signed, or U8.
                (Self::Unsigned(_), Self::Signed(_)) | (Self::Signed(_), Self::Unsigned(_)) => {
                    Err(TypeError::Converge)
                }
                (Self::Signed(bitsl), Self::Signed(bitsr)) => {
                    Ok(Self::Signed(std::cmp::max(*bitsl, *bitsr)))
                }
                (Self::Unsigned(bitsl), Self::Unsigned(bitsr)) => {
                    Ok(Self::Unsigned(std::cmp::max(*bitsl, *bitsr)))
                }
                _ => Err(TypeError::Converge),
            }
        }
    }

    fn generate_test_graph() -> Graph<AbstractTypes, (EntityId<usize>, EntityId<usize>)> {
        let mut graph: Graph<AbstractTypes, (EntityId<usize>, EntityId<usize>)> = Graph::new();
        let nodes = vec![
            AbstractTypes::Any,
            AbstractTypes::Numeric,
            AbstractTypes::Integer,
            AbstractTypes::Signed(8),
            AbstractTypes::Unsigned(8), // unsigned int a
            AbstractTypes::Unsigned(8), // unsigned int b
            AbstractTypes::String,
        ];

        for node in nodes.into_iter() {
            graph.mut_add_node(node);
        }

        let directed_edges: Vec<(EntityId<usize>, EntityId<usize>)> = vec![
            (EntityId::new(0), EntityId::new(1)),
            (EntityId::new(0), EntityId::new(6)),
            (EntityId::new(1), EntityId::new(2)),
            (EntityId::new(2), EntityId::new(3)),
            (EntityId::new(2), EntityId::new(4)),
            (EntityId::new(2), EntityId::new(5)),
        ];

        for edge in directed_edges.into_iter() {
            graph.mut_add_directed_edge(edge.0, edge.1);
        }

        graph
    }

    #[test]
    fn graph_should_add_nodes() {
        let mut graph: Graph<AbstractTypes, (EntityId<usize>, EntityId<usize>)> = Graph::new();

        assert_eq!(EntityId::new(0), graph.mut_add_node(AbstractTypes::Any))
    }

    #[test]
    fn node_with_no_edges_should_return_none() {
        let mut graph: Graph<AbstractTypes, (EntityId<usize>, EntityId<usize>)> = Graph::new();
        let entity_key = graph.mut_add_node(AbstractTypes::Any);

        assert_eq!(None, graph.upper_bounds(entity_key))
    }

    #[test]
    fn node_without_directed_edge_to_an_upper_bound_should_return_none() {
        let mut graph: Graph<AbstractTypes, (EntityId<usize>, EntityId<usize>)> = Graph::new();
        graph.mut_add_node(AbstractTypes::Any);

        assert_eq!(None, graph.upper_bounds(EntityId::new(0)))
    }

    #[test]
    fn node_with_directed_edge_to_an_upper_bound_should_return_the_child() {
        let graph = generate_test_graph();

        assert_eq!(
            Some(vec![EntityId::new(2)]),
            graph.upper_bounds(EntityId::new(1))
        )
    }
}
