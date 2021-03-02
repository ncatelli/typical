#[derive(Debug, Default, Clone)]
struct OrderedSet<Idx> {
    v: Vec<Idx>,
    s: std::collections::HashSet<Idx>,
}

impl<Idx> OrderedSet<Idx>
where
    Idx: Eq + std::hash::Hash + Clone + Copy,
{
    fn insert(&mut self, value: Idx) -> Option<Idx> {
        if self.s.insert(value.clone()) {
            self.v.push(value);
            Some(value)
        } else {
            None
        }
    }

    fn iter(&self) -> std::slice::Iter<Idx> {
        self.v.iter()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Graph<Idx>
where
    Idx: Into<usize> + From<usize>,
{
    upstream_sets: Vec<OrderedSet<Idx>>,
    downstream_sets: Vec<OrderedSet<Idx>>,
}

impl<Idx> Graph<Idx>
where
    Idx: Clone + Copy + Eq + std::hash::Hash + Default + Into<usize> + From<usize>,
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
            if self.downstream_sets[lhs.into()].insert(rhs).is_some() {
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
