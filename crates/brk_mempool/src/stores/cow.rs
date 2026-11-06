//! Copy-on-write containers. Cloning one shares everything; a write copies only the shard or
//! chunk it lands in, and only while a clone still holds it. A publication (a clone) then costs
//! what changed since the last one, not the whole pool, and readers keep lock-free immutable
//! views.

use std::{
    collections::{HashMap, VecDeque},
    hash::{BuildHasher, Hash},
    sync::Arc,
};

/// Shards a [`CowMap`] splits into: at 300k entries, about 75 each. Measured at 300k transactions,
/// with 400 changing a cycle: 4,096 shards and 64-element chunks publish in ~0.4 ms, write the
/// cycle in ~3.7 ms and free an old publication in ~1 ms (2,048 or 8,192 shards, or 256-element
/// chunks, cost more), against 18, 0 and 7 ms copying whole maps.
const SHARD_BITS: u32 = 12;
const SHARDS: usize = 1 << SHARD_BITS;
/// Elements a [`CowVec`] or [`CowQueue`] chunk holds.
const CHUNK: usize = 64;

/// A hash map in [`SHARDS`] shards behind `Arc`s, picked by middle bits of the key's hash, its
/// halves folded (each shard's own table keeps the low bits for buckets and the top ones for its
/// tags). A lookup is
/// the shard's one probe; a write copies its shard if a clone shares it.
pub struct CowMap<K, V, S> {
    shards: Box<[Arc<HashMap<K, V, S>>]>,
    hasher: S,
}

impl<K, V, S: Clone> Clone for CowMap<K, V, S> {
    fn clone(&self) -> Self {
        Self {
            shards: self.shards.clone(),
            hasher: self.hasher.clone(),
        }
    }
}

impl<K, V, S: BuildHasher + Clone + Default> Default for CowMap<K, V, S> {
    fn default() -> Self {
        let hasher = S::default();
        Self {
            shards: (0..SHARDS)
                .map(|_| Arc::new(HashMap::with_hasher(hasher.clone())))
                .collect(),
            hasher,
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone, S: BuildHasher + Clone> CowMap<K, V, S> {
    fn shard(&self, key: &K) -> usize {
        // The halves folded, then middle bits: a 32-bit hasher's (zero-extended) still spreads.
        let hash = self.hasher.hash_one(key);
        ((hash ^ (hash >> 32)) >> 16) as usize & (SHARDS - 1)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.shards[self.shard(key)].get(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// `key`'s shard, to write: copied first if a clone shares it.
    pub fn shard_mut(&mut self, key: &K) -> &mut HashMap<K, V, S> {
        let shard = self.shard(key);
        Arc::make_mut(&mut self.shards[shard])
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        // Only a present key's shard is copied.
        if !self.contains_key(key) {
            return None;
        }
        self.shard_mut(key).get_mut(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.shard_mut(&key).insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if !self.contains_key(key) {
            return None;
        }
        self.shard_mut(key).remove(key)
    }

    /// O(shards).
    pub fn len(&self) -> usize {
        self.shards.iter().map(|shard| shard.len()).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.shards.iter().flat_map(|shard| shard.iter())
    }
}

/// A vector in [`CHUNK`]-element chunks behind `Arc`s: a write copies its chunk if a clone
/// shares it.
pub struct CowVec<T> {
    chunks: Vec<Arc<Vec<T>>>,
    len: usize,
}

impl<T> Clone for CowVec<T> {
    fn clone(&self) -> Self {
        Self {
            chunks: self.chunks.clone(),
            len: self.len,
        }
    }
}

impl<T> Default for CowVec<T> {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            len: 0,
        }
    }
}

impl<T: Clone> CowVec<T> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.chunks.get(index / CHUNK)?.get(index % CHUNK)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        Arc::make_mut(&mut self.chunks[index / CHUNK]).get_mut(index % CHUNK)
    }

    pub fn last(&self) -> Option<&T> {
        self.chunks.last()?.last()
    }

    pub fn push(&mut self, value: T) {
        if self.len.is_multiple_of(CHUNK) {
            self.chunks.push(Arc::new(Vec::with_capacity(CHUNK)));
        }
        Arc::make_mut(self.chunks.last_mut().expect("a chunk to push into")).push(value);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        let last = self.chunks.last_mut()?;
        let value = Arc::make_mut(last).pop();
        if last.is_empty() {
            self.chunks.pop();
        }
        self.len -= usize::from(value.is_some());
        value
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.chunks.iter().flat_map(|chunk| chunk.iter())
    }
}

/// An insertion-ordered map with dense positions (`IndexMap`'s `swap_remove` semantics): the
/// entries in a [`CowVec`], their positions in a [`CowMap`].
pub struct CowIndexMap<K, V, S> {
    positions: CowMap<K, u32, S>,
    entries: CowVec<(K, V)>,
}

impl<K: Clone, V: Clone, S: Clone> Clone for CowIndexMap<K, V, S> {
    fn clone(&self) -> Self {
        Self {
            positions: self.positions.clone(),
            entries: self.entries.clone(),
        }
    }
}

impl<K, V, S: BuildHasher + Clone + Default> Default for CowIndexMap<K, V, S> {
    fn default() -> Self {
        Self {
            positions: CowMap::default(),
            entries: CowVec::default(),
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone, S: BuildHasher + Clone> CowIndexMap<K, V, S> {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let position = *self.positions.get(key)?;
        self.entries.get(position as usize).map(|(_, value)| value)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.positions.contains_key(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let position = *self.positions.get(key)?;
        self.entries
            .get_mut(position as usize)
            .map(|(_, value)| value)
    }

    /// The entry at `index` in dense order.
    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        self.entries.get(index).map(|(key, value)| (key, value))
    }

    pub fn last(&self) -> Option<(&K, &V)> {
        self.entries.last().map(|(key, value)| (key, value))
    }

    /// Appends `key`, which must be absent.
    pub fn insert(&mut self, key: K, value: V) {
        let position = u32::try_from(self.entries.len()).expect("positions fit 32 bits");
        let previous = self.positions.insert(key.clone(), position);
        debug_assert!(previous.is_none(), "inserted keys are new");
        self.entries.push((key, value));
    }

    /// Removes `key`, moving the last entry into its place: its position, key and value.
    pub fn swap_remove_full(&mut self, key: &K) -> Option<(usize, K, V)> {
        let position = self.positions.remove(key)? as usize;
        let last = self.entries.pop().expect("a positioned key has an entry");
        if position == self.entries.len() {
            return Some((position, last.0, last.1));
        }
        self.positions.insert(last.0.clone(), position as u32);
        let slot = self.entries.get_mut(position).expect("an earlier position");
        let (key, value) = std::mem::replace(slot, last);
        Some((position, key, value))
    }

    /// `(key, value)` pairs in dense order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().map(|(key, value)| (key, value))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|(_, value)| value)
    }
}

/// A first-in first-out queue in [`CHUNK`]-element chunks behind `Arc`s: pushing copies the
/// back chunk if a clone shares it, popping shares the front one (only an offset moves).
pub struct CowQueue<T> {
    chunks: VecDeque<Arc<Vec<T>>>,
    /// The front chunk's first element still queued.
    head: usize,
    len: usize,
}

impl<T> Clone for CowQueue<T> {
    fn clone(&self) -> Self {
        Self {
            chunks: self.chunks.clone(),
            head: self.head,
            len: self.len,
        }
    }
}

impl<T> Default for CowQueue<T> {
    fn default() -> Self {
        Self {
            chunks: VecDeque::new(),
            head: 0,
            len: 0,
        }
    }
}

impl<T: Clone> CowQueue<T> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn front(&self) -> Option<&T> {
        self.chunks.front()?.get(self.head)
    }

    pub fn push_back(&mut self, value: T) {
        if self.chunks.back().is_none_or(|chunk| chunk.len() == CHUNK) {
            self.chunks.push_back(Arc::new(Vec::with_capacity(CHUNK)));
        }
        Arc::make_mut(self.chunks.back_mut().expect("a chunk to push into")).push(value);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let front = self.chunks.front()?;
        let value = front.get(self.head)?.clone();
        self.head += 1;
        self.len -= 1;
        if self.head == front.len() {
            self.chunks.pop_front();
            self.head = 0;
        }
        Some(value)
    }

    /// Front to back; `.rev()` back to front.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
        let head = self.head;
        self.chunks
            .iter()
            .enumerate()
            .flat_map(move |(index, chunk)| chunk[if index == 0 { head } else { 0 }..].iter())
    }

    /// Keeps what `keep` accepts, in order: O(len), the queue rebuilt.
    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        let kept: Vec<T> = self.iter().filter(|value| keep(value)).cloned().collect();
        *self = Self::default();
        for value in kept {
            self.push_back(value);
        }
    }
}
