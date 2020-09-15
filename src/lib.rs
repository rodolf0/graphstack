use std::collections::HashMap;

pub struct GraphStack<T> {
    /// Holds the elements inserted into the GraphStack.
    items: Vec<T>,

    /// In a stack, each element sits on top of another.
    /// In a graph-stack, each item may have multiple ancestors.
    /// The indexes in `ancestors` direclty map to items in `items`.
    ancestors: HashMap<usize, Vec<usize>>,
}

impl<T> GraphStack<T> {
    pub fn new() -> Self {
        GraphStack {
            items: Vec::new(),
            ancestors: HashMap::new(),
        }
    }

    /// Adds an element to the graph-stack and returns an item-id for it.
    /// This `id` can later be used to add ancestors for this item.
    pub fn push(&mut self, value: T, ancestors: &[usize]) -> usize {
        // Check that each ancestor is valid
        if ancestors.iter().any(|a| *a >= self.items.len()) {
            panic!(
                "Invalid ancestors. GS size={}, ancestors={:#?}",
                self.items.len(),
                ancestors
            );
        }
        self.items.push(value);
        let item_id = self.items.len() - 1;
        self.ancestors
            .insert(item_id, ancestors.iter().cloned().collect());
        item_id
    }

    pub fn add_ancestors(&mut self, id: usize, ancestors: &[usize]) {
        if !self.ancestors.contains_key(&id) {
            panic!("Invalid ancestor id={}", id);
        }
        // TODO: detect cycles
        self.ancestors.entry(id).or_default().extend(ancestors);
    }

    /// Build an iterator over the stacks encoded by this GraphStack.
    /// A `start_item` is required because there may be multiple top items.
    pub fn stacks(&self, start_item: usize) -> Stacks<T> {
        Stacks::new(&self, start_item)
    }
}

/// A cursor for keeping track when iterating over a GraphStack.
#[derive(Debug)]
struct Cursor {
    item: usize,
    ancestor: usize,
}

/// An iterator to retrieve stacks encoded in GraphStack.
pub struct Stacks<'a, T> {
    /// Need a cursor for each item in the GraphStack to track
    /// which of its ancestors is currently being traversed.
    cursors: Vec<Cursor>,

    /// This is where the current stack to be returned will be unpacked.
    unstack: Vec<&'a T>,

    /// A reference to the GraphStack that this iterator is traversing.
    gs: &'a GraphStack<T>,
}

impl<'a, T> Stacks<'a, T> {
    fn new(gs: &'a GraphStack<T>, start_item: usize) -> Self {
        Stacks {
            cursors: vec![Cursor {
                item: start_item,
                ancestor: 0,
            }],
            unstack: vec![&gs.items[start_item]],
            gs,
        }
    }
}

impl<'a, T> Iterator for Stacks<'a, T> {
    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursors.is_empty() {
            return None;
        }

        // Build a snapshot of the stack pointed to by current cursors
        while let Some(cursor) = self.cursors.last() {
            let ref item_ancestors = self.gs.ancestors[&cursor.item];
            // Is cursor at the bottom of the stack, or has more depth?
            if item_ancestors.is_empty() {
                break;
            }
            let prev_item_id = item_ancestors[cursor.ancestor];
            self.unstack.push(&self.gs.items[prev_item_id]);
            self.cursors.push(Cursor {
                item: prev_item_id,
                ancestor: 0,
            });
        }
        let stack_snapshot = self.unstack.clone();

        // Advance iterator: find the cursor to advance depth-first
        while let Some(cursor) = self.cursors.last_mut() {
            let num_item_ancestors = self.gs.ancestors[&cursor.item].len();
            if cursor.ancestor + 1 < num_item_ancestors {
                cursor.ancestor += 1;
                break;
            }
            self.cursors.pop();
        }
        // keep the part of the stack that is common for other ancestors
        self.unstack.truncate(self.cursors.len());

        Some(stack_snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::GraphStack;
    use std::collections::HashMap;

    #[test]
    fn check_iterator() {
        // a - b - c - e - f - g - h
        //      \ d -/------/
        let mut gs = GraphStack::new();
        let idmap: HashMap<_, _> = ["a", "b", "c", "d", "e", "f", "g", "h"]
            .iter()
            .cloned()
            .map(|value| (value, gs.push(value, &[])))
            .collect();

        gs.add_ancestors(idmap["b"], &[idmap["a"]]);
        gs.add_ancestors(idmap["c"], &[idmap["b"]]);
        gs.add_ancestors(idmap["d"], &[idmap["b"]]);
        gs.add_ancestors(idmap["e"], &[idmap["c"], idmap["d"]]);
        gs.add_ancestors(idmap["f"], &[idmap["e"]]);
        gs.add_ancestors(idmap["g"], &[idmap["d"], idmap["f"]]);
        gs.add_ancestors(idmap["h"], &[idmap["g"]]);
        let mut it = gs.stacks(idmap["h"]);
        assert_eq!(it.next().unwrap(), vec![&"h", &"g", &"d", &"b", &"a"]);
        assert_eq!(
            it.next().unwrap(),
            vec![&"h", &"g", &"f", &"e", &"c", &"b", &"a"]
        );
        assert_eq!(
            it.next().unwrap(),
            vec![&"h", &"g", &"f", &"e", &"d", &"b", &"a"]
        );
        assert!(it.next().is_none());
    }

    #[test]
    fn disjoint_stacks() {
        // a - b - c
        // d - e
        let mut gs = GraphStack::new();
        let idmap: HashMap<_, _> = ["a", "b", "c", "d", "e"]
            .iter()
            .cloned()
            .map(|value| (value, gs.push(value, &[])))
            .collect();

        gs.add_ancestors(idmap["b"], &[idmap["a"]]);
        gs.add_ancestors(idmap["c"], &[idmap["b"]]);
        // disjoint stack
        gs.add_ancestors(idmap["e"], &[idmap["d"]]);

        let mut it = gs.stacks(idmap["e"]);
        assert_eq!(it.next().unwrap(), vec![&"e", &"d"]);
        assert!(it.next().is_none());

        let mut it = gs.stacks(idmap["c"]);
        assert_eq!(it.next().unwrap(), vec![&"c", &"b", &"a"]);
        assert!(it.next().is_none());
    }

    #[test]
    fn x_stack() {
        // a - b - c
        // d /  \ e
        let mut gs = GraphStack::new();
        let idmap: HashMap<_, _> = ["a", "b", "c", "d", "e"]
            .iter()
            .cloned()
            .map(|value| (value, gs.push(value, &[])))
            .collect();

        gs.add_ancestors(idmap["b"], &[idmap["a"], idmap["d"]]);
        gs.add_ancestors(idmap["c"], &[idmap["b"]]);
        gs.add_ancestors(idmap["e"], &[idmap["b"]]);

        let mut it = gs.stacks(idmap["e"]);
        assert_eq!(it.next().unwrap(), vec![&"e", &"b", &"a"]);
        assert_eq!(it.next().unwrap(), vec![&"e", &"b", &"d"]);
        assert!(it.next().is_none());

        let mut it = gs.stacks(idmap["c"]);
        assert_eq!(it.next().unwrap(), vec![&"c", &"b", &"a"]);
        assert_eq!(it.next().unwrap(), vec![&"c", &"b", &"d"]);
        assert!(it.next().is_none());

        let mut it = gs.stacks(idmap["b"]);
        assert_eq!(it.next().unwrap(), vec![&"b", &"a"]);
        assert_eq!(it.next().unwrap(), vec![&"b", &"d"]);
        assert!(it.next().is_none());
    }

    // TODO: test case for adding a cycle
    // create a cycle
    // gs.add_ancestors(idmap["a"], &[idmap["h"]]);
}
