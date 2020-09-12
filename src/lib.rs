use std::collections::{HashMap, HashSet};

pub struct GraphStack<'a, T> {
    data: Vec<&'a T>,
    ancestors: HashMap<usize, HashSet<usize>>,
}

impl<'a, T> GraphStack<'a, T> {
    pub fn new() -> Self {
        GraphStack {
            data: Vec::new(),
            ancestors: HashMap::new(),
        }
    }

    /// Adds an element to the graph-stack
    pub fn push(&mut self, value: &'a T, ancestors: &[usize]) -> usize {
        // Check that each ancestor is valid
        if ancestors.iter().any(|a| *a >= self.data.len()) {
            panic!(
                "Invalid ancestors. GS size={}, ancestors={:#?}",
                self.data.len(),
                ancestors
            );
        }
        self.data.push(value);
        let id = self.data.len();
        self.ancestors
            .insert(id, ancestors.iter().cloned().collect());
        id
    }

    pub fn stacks() {}
}

#[cfg(test)]
mod tests {
    use super::GraphStack;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn simple_gs() {
        let mut gs = GraphStack::new();
        let items: HashMap<_, HashSet<usize>> = ["a", "b", "c", "d", "e", "f", "g", "h"]
            .iter()
            .cloned()
            .map(|i| (i.to_owned(), HashSet::new()))
            .collect();

        gs.push(&items["a"], &[]);
        gs.push(&items["b"], &[0]);
        gs.push(&items["c"], &[1]);
        gs.push(&items["d"], &[1]);
        gs.push(&items["e"], &[2, 3]);
        gs.push(&items["f"], &[4]);
        gs.push(&items["g"], &[3, 5]);
        gs.push(&items["h"], &[6]);
    }
}
