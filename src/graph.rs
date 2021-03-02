#[derive(Debug, Default, Clone)]
struct OrderedSet<T> {
    v: Vec<T>,
    s: std::collections::HashSet<T>,
}

impl<T: Eq + std::hash::Hash + Clone> OrderedSet<T> {
    fn insert(&mut self, value: T) -> bool {
        if self.s.insert(value.clone()) {
            self.v.push(value);
            true
        } else {
            false
        }
    }

    fn iter(&self) -> std::slice::Iter<T> {
        self.v.iter()
    }
}

type ID = usize;
#[derive(Debug, Default, Clone)]
pub struct Graph<ID>
where
    ID: Into<usize> + From<usize>,
{
    upstream_sets: Vec<OrderedSet<ID>>,
    downstream_sets: Vec<OrderedSet<ID>>,
}

impl Graph<ID>
where
    ID: Into<usize> + From<usize>,
{
    pub fn add_node_mut(&mut self) -> ID {
        let i = ID::from(self.upstream_sets.len());

        self.upstream_sets.push(OrderedSet::default());
        self.downstream_sets.push(OrderedSet::default());
        i
    }

    pub fn add_node(mut self) -> (ID, Self) {
        let i = ID::from(self.upstream_sets.len());

        self.upstream_sets.push(OrderedSet::default());
        self.downstream_sets.push(OrderedSet::default());
        (i, self)
    }

    pub fn add_edge_mut(&mut self, lhs: ID, rhs: ID, mut out: Vec<(ID, ID)>) -> Vec<(ID, ID)> {
        let mut work = vec![(lhs, rhs)];

        while let Some((lhs, rhs)) = work.pop() {
            let (lhs, rhs) = (usize::from(lhs), usize::from(rhs));

            // Attempt to insert the rhs into the downstream_set
            if self.downstream_sets[lhs].insert(rhs) {
                self.upstream_sets[rhs].insert(lhs);
                // Inform the caller that a new edge was added
                out.push((lhs, rhs));

                for &lhs2 in self.upstream_sets[lhs].iter() {
                    work.push((lhs2, rhs));
                }
                for &rhs2 in self.downstream_sets[rhs].iter() {
                    work.push((lhs, rhs2));
                }
            }
        }

        out
    }

    pub fn add_edge(mut self, lhs: ID, rhs: ID) -> (Self, Vec<(ID, ID)>) {
        let mut work = vec![(lhs, rhs)];
        let mut out = Vec::<(ID, ID)>::new();

        while let Some((lhs, rhs)) = work.pop() {
            let (lhs, rhs) = (usize::from(lhs), usize::from(rhs));

            // Attempt to insert the rhs into the downstream_set
            if self.downstream_sets[lhs].insert(rhs) {
                self.upstream_sets[rhs].insert(lhs);
                // Inform the caller that a new edge was added
                out.push((lhs, rhs));

                for &lhs2 in self.upstream_sets[lhs].iter() {
                    work.push((lhs2, rhs));
                }
                for &rhs2 in self.downstream_sets[rhs].iter() {
                    work.push((lhs, rhs2));
                }
            }
        }

        (self, out)
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

        let mut new_edges: Vec<(usize, usize)> = [(0, 3), (1, 3), (2, 3), (3, 4)]
            .iter()
            .fold((graph, vec![]), |(g, _), (upstream, downstream)| {
                g.add_edge(*upstream, *downstream)
            })
            .1;

        let mut expected: Vec<(usize, usize)> = [0, 1, 2, 3].iter().map(|&lhs| (lhs, 4)).collect();

        new_edges.sort_unstable();
        expected.sort_unstable();
        assert_eq!(expected, new_edges);
    }
}
