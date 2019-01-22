//! This is a "monotonic HashMap": A HashMap that, when shared, can be pushed to but not
//! otherwise mutated.  We also Box items in the map. This means we can safely provide
//! shared references into existing items in the HashMap, because they will not be dropped
//! (from being removed) or moved (because they are boxed).
//! The API is is completely tailored to what `memory.rs` needs. It is still in
//! a separate file to minimize the amount of code that has to care about the unsafety.

use std::collections::hash_map::Entry;
use std::cell::RefCell;
use std::hash::Hash;
use std::borrow::Borrow;

use rustc_data_structures::fx::FxHashMap;

use crate::AllocMap;

#[derive(Debug, Clone)]
pub struct MonoHashMap<K: Hash + Eq, V>(RefCell<FxHashMap<K, Box<V>>>);

impl<K: Hash + Eq, V> MonoHashMap<K, V> {
    /// This function exists for priroda to be able to iterate over all evaluator memory
    ///
    /// The function is somewhat roundabout with the closure argument because internally the
    /// `MonoHashMap` uses a `RefCell`. When iterating over the `HashMap` inside the `RefCell`,
    /// we need to keep a borrow to the `HashMap` inside the iterator. The borrow is only alive
    /// as long as the `Ref` returned by `RefCell::borrow()` is alive. So we can't return the
    /// iterator, as that would drop the `Ref`. We can't return both, as it's not possible in Rust
    /// to have a struct/tuple with a field that refers to another field.
    pub fn iter<T>(&self, f: impl FnOnce(&mut dyn Iterator<Item=(&K, &V)>) -> T) -> T {
        f(&mut self.0.borrow().iter().map(|(k, v)| (k, &**v)))
    }
}

impl<K: Hash + Eq, V> Default for MonoHashMap<K, V> {
    fn default() -> Self {
        MonoHashMap(RefCell::new(Default::default()))
    }
}

impl<K: Hash + Eq, V> AllocMap<K, V> for MonoHashMap<K, V> {
    #[inline(always)]
    fn contains_key<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> bool
        where K: Borrow<Q>
    {
        self.0.get_mut().contains_key(k)
    }

    #[inline(always)]
    fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        self.0.get_mut().insert(k, Box::new(v)).map(|x| *x)
    }

    #[inline(always)]
    fn remove<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<V>
        where K: Borrow<Q>
    {
        self.0.get_mut().remove(k).map(|x| *x)
    }

    #[inline(always)]
    fn filter_map_collect<T>(&self, mut f: impl FnMut(&K, &V) -> Option<T>) -> Vec<T> {
        self.0.borrow()
            .iter()
            .filter_map(move |(k, v)| f(k, &*v))
            .collect()
    }

    /// The most interesting method: Providing a shared ref without
    /// holding the `RefCell` open, and inserting new data if the key
    /// is not used yet.
    /// `vacant` is called if the key is not found in the map;
    /// if it returns a reference, that is used directly, if it
    /// returns owned data, that is put into the map and returned.
    #[inline(always)]
    fn get_or<E>(
        &self,
        k: K,
        vacant: impl FnOnce() -> Result<V, E>
    ) -> Result<&V, E> {
        let val: *const V = match self.0.borrow_mut().entry(k) {
            Entry::Occupied(entry) => &**entry.get(),
            Entry::Vacant(entry) => &**entry.insert(Box::new(vacant()?)),
        };
        // This is safe because `val` points into a `Box`, that we know will not move and
        // will also not be dropped as long as the shared reference `self` is live.
        unsafe { Ok(&*val) }
    }

    #[inline(always)]
    fn get_mut_or<E>(
        &mut self,
        k: K,
        vacant: impl FnOnce() -> Result<V, E>
    ) -> Result<&mut V, E>
    {
        match self.0.get_mut().entry(k) {
            Entry::Occupied(e) => Ok(e.into_mut()),
            Entry::Vacant(e) => {
                let v = vacant()?;
                Ok(e.insert(Box::new(v)))
            }
        }
    }
}
