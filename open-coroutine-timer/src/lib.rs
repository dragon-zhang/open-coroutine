#![deny(
    // The following are allowed by default lints according to
    // https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    anonymous_parameters,
    bare_trait_objects,
    box_pointers,
    elided_lifetimes_in_paths,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    warnings, // treat all wanings as errors

    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    // clippy::nursery, // It's still under development
    clippy::cargo,
)]
#![allow(
    // Some explicitly allowed Clippy lints, must have clear reason to allow
    clippy::blanket_clippy_restriction_lints, // allow clippy::restriction
    clippy::implicit_return, // actually omitting the return keyword is idiomatic Rust code
    clippy::module_name_repetitions, // repeation of module name in a struct name is not big deal
    clippy::multiple_crate_versions, // multi-version dependency crates is not able to fix
    clippy::panic_in_result_fn,
    clippy::shadow_same, // Not too much bad
    clippy::shadow_reuse, // Not too much bad
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::indexing_slicing,
    clippy::separated_literal_suffix, // conflicts with clippy::unseparated_literal_suffix
    clippy::single_char_lifetime_names,
)]

//! Associate `VecDeque` with `timestamps`.

use std::collections::vec_deque::{Iter, IterMut};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// get the current wall clock in ns
///
/// # Panics
/// if the time is before `UNIX_EPOCH`
#[must_use]
pub fn now() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("1970-01-01 00:00:00 UTC was {} seconds ago!")
            .as_nanos(),
    )
    .unwrap_or(u64::MAX)
}

/// current ns time add `dur`.
#[must_use]
pub fn get_timeout_time(dur: Duration) -> u64 {
    u64::try_from(dur.as_nanos())
        .map(|d| d.saturating_add(now()))
        .unwrap_or(u64::MAX)
}

/// A queue for managing multiple entries under a specified timestamp.
#[derive(Debug, Eq, PartialEq)]
pub struct TimerEntry<T> {
    timestamp: u64,
    inner: VecDeque<T>,
}

impl<T> TimerEntry<T> {
    /// Creates an empty deque.
    #[must_use]
    pub fn new(timestamp: u64) -> Self {
        TimerEntry {
            timestamp,
            inner: VecDeque::new(),
        }
    }

    /// Returns the number of elements in the deque.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the deque is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the timestamp.
    #[must_use]
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Removes the first element and returns it, or `None` if the deque is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    /// Appends an element to the back of the deque.
    pub fn push_back(&mut self, t: T) {
        self.inner.push_back(t);
    }

    /// Removes and returns the `t` from the deque.
    /// Whichever end is closer to the removal point will be moved to make
    /// room, and all the affected elements will be moved to new positions.
    /// Returns `None` if `t` not found.
    pub fn remove(&mut self, t: &T) -> Option<T>
    where
        T: Ord,
    {
        let index = self
            .inner
            .binary_search_by(|x| x.cmp(t))
            .unwrap_or_else(|x| x);
        self.inner.remove(index)
    }

    /// Returns a front-to-back iterator that returns mutable references.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.inner.iter_mut()
    }

    /// Returns a front-to-back iterator.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }
}

/// A queue for managing multiple `TimerEntry`.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct TimerList<T>(VecDeque<TimerEntry<T>>);

impl<T> Default for TimerList<T> {
    fn default() -> Self {
        TimerList(VecDeque::new())
    }
}

impl<T> TimerList<T> {
    /// Returns the number of elements in the deque.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Inserts an element at `timestamp` within the deque, shifting all elements
    /// with indices greater than or equal to `timestamp` towards the back.
    pub fn insert(&mut self, timestamp: u64, t: T) {
        let index = self
            .0
            .binary_search_by(|x| x.timestamp.cmp(&timestamp))
            .unwrap_or_else(|x| x);
        if let Some(entry) = self.0.get_mut(index) {
            entry.push_back(t);
        } else {
            let mut entry = TimerEntry::new(timestamp);
            entry.push_back(t);
            self.0.insert(index, entry);
        }
    }

    /// Provides a reference to the front element, or `None` if the deque is empty.
    #[must_use]
    pub fn front(&self) -> Option<&TimerEntry<T>> {
        self.0.front()
    }

    /// Removes the first element and returns it, or `None` if the deque is empty.
    pub fn pop_front(&mut self) -> Option<TimerEntry<T>> {
        self.0.pop_front()
    }

    /// Returns `true` if the deque is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        for entry in &self.0 {
            if !entry.is_empty() {
                return false;
            }
        }
        true
    }

    /// Provides a mutable reference to the entry at the given `timestamp`.
    pub fn get_entry(&mut self, timestamp: &u64) -> Option<&mut TimerEntry<T>> {
        let index = self
            .0
            .binary_search_by(|x| x.timestamp.cmp(timestamp))
            .unwrap_or_else(|x| x);
        self.0.get_mut(index)
    }

    /// Removes and returns the element at `timestamp` from the deque.
    /// Whichever end is closer to the removal point will be moved to make
    /// room, and all the affected elements will be moved to new positions.
    /// Returns `None` if `timestamp` is out of bounds.
    pub fn remove(&mut self, timestamp: &u64) -> Option<TimerEntry<T>> {
        let index = self
            .0
            .binary_search_by(|x| x.timestamp.cmp(timestamp))
            .unwrap_or_else(|x| x);
        self.0.remove(index)
    }

    /// Returns a front-to-back iterator that returns mutable references.
    pub fn iter_mut(&mut self) -> IterMut<'_, TimerEntry<T>> {
        self.0.iter_mut()
    }

    /// Returns a front-to-back iterator.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, TimerEntry<T>> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert!(now() > 0);
    }

    #[test]
    fn timer_list() {
        let mut list = TimerList::default();
        assert_eq!(list.len(), 0);
        list.insert(1, String::from("data is typed"));
        assert_eq!(list.len(), 1);

        let mut entry = list.pop_front().unwrap();
        assert_eq!(entry.len(), 1);
        let string = entry.pop_front().unwrap();
        assert_eq!(string, String::from("data is typed"));
    }
}
