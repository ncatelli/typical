use std::fmt::Debug;

#[derive(Default, Clone)]
/// OrderedSet maintains a consistent order of items determined by the sequence
/// that elements were added to the set.
struct OrderedSet<T> {
    v: Vec<T>,
    s: std::collections::HashSet<T>,
}

impl<T> OrderedSet<T>
where
    T: Eq + std::hash::Hash + Clone + Copy,
{
    /// If an element doesn't currently exist in a set, it is appended to the
    /// end of the set and true is returned.
    fn insert(&mut self, value: T) -> bool {
        if self.s.insert(value.clone()) {
            self.v.push(value);
            true
        } else {
            false
        }
    }
}

impl<T> OrderedSet<T>
where
    T: Clone + Copy,
{
    fn iter(&self) -> std::slice::Iter<T> {
        self.v.iter()
    }
}

impl<T> std::fmt::Debug for OrderedSet<T>
where
    T: std::fmt::Debug + Clone + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OrderedSet{{{}}}",
            self.clone()
                .iter()
                .map(|t| format!("{:?}", t))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

/// Graph represents a series of value IDs as upstream and downstream sets
/// where upstream sets map all the nodes that have edges to a given node
/// and downsets that map all edges from a given node.
#[derive(Debug, Default, Clone)]
pub struct Graph<Idx>
where
    Idx: Into<usize> + From<usize> + Debug + Copy,
{
    /// maps all nodes that have an edge _to_ a given node.
    upstream_sets: Vec<OrderedSet<Idx>>,
    /// maps all nodes that have an edge _from_ a given node.
    downstream_sets: Vec<OrderedSet<Idx>>,
}

impl<Idx> Graph<Idx>
where
    Idx: Clone + Copy + Eq + std::hash::Hash + Default + Into<usize> + From<usize> + Debug,
{
    /// Adds a new node in place by reference, returning the Id of the node.
    pub fn add_node_mut(&mut self) -> Idx {
        self.upstream_sets.push(OrderedSet::default());
        self.downstream_sets.push(OrderedSet::default());

        Idx::from(self.upstream_sets.len() - 1)
    }

    /// Adds a new node by value, returning the modified instance of itself.
    #[allow(dead_code)]
    pub fn add_node(mut self) -> (Self, Idx) {
        let new_id = self.add_node_mut();
        (self, new_id)
    }

    /// Adds a new edge, updating existing edges to maintain transitivity.
    pub fn add_edge_mut(&mut self, lhs: Idx, rhs: Idx) -> Vec<(Idx, Idx)> {
        let mut work = vec![(lhs, rhs)];
        let mut new_edges = Vec::new();

        while let Some((lhs, rhs)) = work.pop() {
            // Attempt to insert the rhs into the downstream_set
            if self.downstream_sets[lhs.into()].insert(rhs) {
                self.upstream_sets[rhs.into()].insert(lhs);
                // Inform the caller that a new edge was added
                new_edges.push((lhs, rhs));

                for &lhs2 in self.upstream_sets[lhs.into()].iter() {
                    work.push((lhs2, rhs));
                }
                for &rhs2 in self.downstream_sets[rhs.into()].iter() {
                    work.push((lhs, rhs2));
                }
            }
        }

        new_edges
    }

    /// Adds a new edge by value returning the modified instance of the graph and all new edges.
    #[allow(dead_code)]
    pub fn add_edge(mut self, lhs: Idx, rhs: Idx) -> (Self, Vec<(Idx, Idx)>) {
        let new_edges = self.add_edge_mut(lhs, rhs);

        (self, new_edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_should_resolve_transitivity() {
        let graph = (0..10).fold(Graph::default(), |acc, _| acc.add_node().0);

        let (_, mut new_edges) = [(0, 3), (1, 3), (2, 3), (3, 4)]
            .iter()
            .fold((graph, vec![]), |(g, _), (upstream, downstream)| {
                g.add_edge(*upstream, *downstream)
            });

        let mut expected: Vec<(_, _)> = [0, 1, 2, 3].iter().map(|&lhs| (lhs, 4)).collect();

        new_edges.sort_unstable();
        expected.sort_unstable();
        assert_eq!(expected, new_edges);
    }
}
