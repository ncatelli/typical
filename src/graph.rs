use std::fmt::Debug;

type NodeId = crate::types::EntityId<usize>;

pub trait Edge: Copy {
    fn new(src: NodeId, dest: NodeId) -> Self;
    fn source(&self) -> NodeId;
    fn destination(&self) -> NodeId;
}

impl Edge for (NodeId, NodeId) {
    fn new(src: NodeId, dest: NodeId) -> Self {
        (src, dest)
    }

    fn source(&self) -> NodeId {
        self.0
    }

    fn destination(&self) -> NodeId {
        self.1
    }
}

enum Node<N> {
    SubType,
    Representative(N),
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

    pub fn add_node(mut self, new_node: N) -> (NodeId, Self) {
        let new_key = self.mut_add_node(new_node);
        (new_key, self)
    }

    pub fn mut_add_parent_binding(&mut self, child: NodeId, parent: NodeId) -> E {
        let parent_to_child = E::new(parent, child);
        self.edges[usize::from(child)].push(parent_to_child);
        self.edges[usize::from(parent)].push(parent_to_child);

        parent_to_child
    }

    pub fn add_parent_binding(mut self, child: NodeId, parent: NodeId) -> (E, Self) {
        let new_edge = self.mut_add_parent_binding(child, parent);
        (new_edge, self)
    }

    pub fn parents(&self, eid: NodeId) -> Option<Vec<NodeId>> {
        let edges: Vec<NodeId> = self.edges[usize::from(eid)]
            .iter()
            .map(|&e| {
                if e.destination() == eid {
                    Some(e.source())
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

    pub fn children(&self, eid: NodeId) -> Option<Vec<NodeId>> {
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

#[cfg(test)]
mod tests {
    use crate::graph::*;
    use crate::types::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum AbstractTypes {
        Any,
        Numeric,
        Integer,
        Unsigned,
        Unsigned8,
        Signed,
        String,
    }

    impl AbstractEntity for AbstractTypes {
        fn unconstrained_type() -> Self {
            Self::Any
        }

        fn arity(&self) -> Option<usize> {
            match self {
                Self::Any => None,
                Self::Numeric => Some(3),
                Self::Integer => Some(2),
                Self::Unsigned => Some(1),
                Self::Unsigned8 => Some(0),
                Self::Signed => Some(1),
                Self::String => Some(0),
            }
        }

        fn converge(&self, other: &Self) -> Result<Self, TypeError> {
            match (self, other) {
                (Self::Any, o) | (o, Self::Any) => Ok(*o),
                (Self::String, Self::String) => Ok(Self::String),
                (Self::String, _) | (_, Self::String) => Err(TypeError::Converge),
                (Self::Numeric, o) | (o, Self::Numeric) => Ok(*o), // o can only be Numeric, Integer, Unsigned, Signed, or U8.
                (Self::Integer, o) | (o, Self::Integer) => Ok(*o), // o can only be  Integer, Unsigned, Signed, or U8.
                (Self::Unsigned, Self::Signed) | (Self::Signed, Self::Unsigned) => {
                    Err(TypeError::Converge)
                }
                (Self::Signed, Self::Signed) => Ok(Self::Signed),
                (Self::Unsigned, o) | (o, Self::Unsigned) => Ok(*o), // o can only be Unsigned or Unsigned8.
                (Self::Unsigned8, Self::Unsigned8) => Ok(Self::Unsigned8),
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
            AbstractTypes::Unsigned,
            AbstractTypes::Signed,
            AbstractTypes::Unsigned8,
            AbstractTypes::String,
        ];

        for node in nodes.into_iter() {
            graph.mut_add_node(node);
        }

        let edges: Vec<(EntityId<usize>, EntityId<usize>)> = vec![
            (EntityId::new(1), EntityId::new(0)),
            (EntityId::new(6), EntityId::new(0)),
            (EntityId::new(2), EntityId::new(1)),
            (EntityId::new(3), EntityId::new(2)),
            (EntityId::new(4), EntityId::new(2)),
            (EntityId::new(5), EntityId::new(3)),
        ];

        for edge in edges.into_iter() {
            graph.mut_add_parent_binding(edge.0, edge.1);
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
        let graph = generate_test_graph();

        assert_eq!(None, graph.children(EntityId::new(1)))
    }

    #[test]
    fn node_without_parent_edge_should_return_none() {
        let mut graph: Graph<AbstractTypes, (EntityId<usize>, EntityId<usize>)> = Graph::new();
        graph.mut_add_node(AbstractTypes::Any);

        assert_eq!(None, graph.children(EntityId::new(0)))
    }

    #[test]
    fn node_with_parent_edge_should_return_none() {
        let graph = generate_test_graph();

        assert_eq!(
            Some(vec![EntityId::new(2)]),
            graph.children(EntityId::new(1))
        )
    }
}
