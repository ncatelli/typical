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

/// Graph
#[derive(Debug, Default, Clone)]
pub struct Graph<Idx>
where
    Idx: Into<usize> + From<usize> + Debug + Copy,
{
    upstream_sets: Vec<OrderedSet<Idx>>,
    downstream_sets: Vec<OrderedSet<Idx>>,
}

impl<Idx> Graph<Idx>
where
    Idx: Clone + Copy + Eq + std::hash::Hash + Default + Into<usize> + From<usize> + Debug,
{
    pub fn add_node_mut(&mut self) -> Idx {
        self.upstream_sets.push(OrderedSet::default());
        self.downstream_sets.push(OrderedSet::default());

        Idx::from(self.upstream_sets.len() - 1)
    }

    #[allow(dead_code)]
    pub fn add_node(mut self) -> (Idx, Self) {
        let new_id = self.add_node_mut();
        (new_id, self)
    }

    pub fn add_edge_mut(
        &mut self,
        lhs: Idx,
        rhs: Idx,
        mut new_edges: Vec<(Idx, Idx)>,
    ) -> Vec<(Idx, Idx)> {
        let mut work = vec![(lhs, rhs)];

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

    #[allow(dead_code)]
    pub fn add_edge(mut self, lhs: Idx, rhs: Idx) -> (Self, Vec<(Idx, Idx)>) {
        let new_edges = self.add_edge_mut(lhs, rhs, Vec::new());

        (self, new_edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_should_resolve_transitivity() {
        let graph = (0..10).fold(Graph::default(), |mut acc, _| {
            acc.add_node_mut();
            acc
        });

        let mut new_edges: Vec<(_, _)> = [(0, 3), (1, 3), (2, 3), (3, 4)]
            .iter()
            .fold((graph, vec![]), |(g, _), (upstream, downstream)| {
                g.add_edge(*upstream, *downstream)
            })
            .1;

        let mut expected: Vec<(_, _)> = [0, 1, 2, 3].iter().map(|&lhs| (lhs, 4)).collect();

        new_edges.sort_unstable();
        expected.sort_unstable();
        assert_eq!(expected, new_edges);
    }
}
