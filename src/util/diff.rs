use std::{collections::HashSet, hash::Hash};

/// Perform a set difference between two iterators.
///
/// Returns the elements that are only in `a`, the elements that are in both
/// `a` and `b`, and the elements that are only in `b`.
pub fn diff<'a, T, A, B>(a: A, b: B) -> (HashSet<T>, HashSet<T>, HashSet<T>)
where
    T: Clone + Eq + Hash + 'a,
    A: Iterator<Item = &'a T>,
    B: Iterator<Item = &'a T>,
{
    let a = a.collect::<HashSet<_>>();
    let b = b.collect::<HashSet<_>>();

    let a_only = a.difference(&b).copied().cloned().collect();
    let both = a.intersection(&b).copied().cloned().collect();
    let b_only = b.difference(&a).copied().cloned().collect();

    (a_only, both, b_only)
}
