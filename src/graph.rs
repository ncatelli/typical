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
pub struct Reachability<ID>
where
    ID: Into<usize> + From<usize>,
{
    upsets: Vec<OrderedSet<ID>>,
    downsets: Vec<OrderedSet<ID>>,
}

impl Reachability<ID>
where
    ID: Into<usize> + From<usize>,
{
    pub fn add_node(&mut self) -> ID {
        let i = ID::from(self.upsets.len());

        self.upsets.push(OrderedSet::default());
        self.downsets.push(OrderedSet::default());
        i
    }

    pub fn add_edge(&mut self, lhs: ID, rhs: ID, mut out: Vec<(ID, ID)>) -> Vec<(ID, ID)> {
        let mut work = vec![(lhs, rhs)];

        while let Some((lhs, rhs)) = work.pop() {
            let (lhs, rhs) = (usize::from(lhs), usize::from(rhs));
            // Insert returns false if the edge is already present
            if !self.downsets[lhs].insert(rhs) {
                continue;
            }
            self.upsets[rhs].insert(lhs);
            // Inform the caller that a new edge was added
            out.push((lhs, rhs));

            for &lhs2 in self.upsets[lhs].iter() {
                work.push((lhs2, rhs));
            }
            for &rhs2 in self.downsets[rhs].iter() {
                work.push((lhs, rhs2));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut r = Reachability::default();
        for _ in 0..10 {
            r.add_node();
        }

        r.add_edge(0, 3, vec![]);
        r.add_edge(1, 3, vec![]);
        r.add_edge(2, 3, vec![]);

        let mut out = r.add_edge(3, 4, vec![]);

        let mut expected = Vec::new();
        for &lhs in &[0, 1, 2, 3] {
            for &rhs in &[4] {
                expected.push((lhs, rhs));
            }
        }

        out.sort_unstable();
        expected.sort_unstable();
        assert_eq!(out, expected);

        println!("{:?}", &out);
    }
}
