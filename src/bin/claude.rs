use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ─── Universal hash family ────────────────────────────────────────────────────
// Each HashFn holds two random seeds (a, b). For a key k it computes:
//   ((a * k + b) mod LARGE_PRIME) mod table_size
// Changing the seeds gives you a new, independent hash function — this is
// exactly the "pick a new function from a universal family" step the paper
// requires on rehash.
#[derive(Clone, Debug)]
struct HashFn {
    a: u64,
    b: u64,
}

impl HashFn {
    // Build a new hash function with pseudo-random seeds derived from a salt.
    // We feed the salt into Rust's DefaultHasher twice (with different inputs)
    // so every (table_index, rehash_round) pair gives different seeds without
    // pulling in an external crate.
    fn new(salt: u64) -> Self {
        let mut h1 = DefaultHasher::new();
        salt.hash(&mut h1);
        let a = h1.finish() | 1; // keep odd so gcd(a, prime) == 1

        let mut h2 = DefaultHasher::new();
        (salt ^ 0xDEAD_BEEF_CAFE_1234).hash(&mut h2);
        let b = h2.finish();

        HashFn { a, b }
    }

    fn apply(&self, key: i32, size: usize) -> usize {
        const P: u64 = 0x1_0000_0003; // a prime just above 2^32
        let k = key as u64;
        (self.a.wrapping_mul(k).wrapping_add(self.b) % P) as usize % size
    }
}

// ─── Cuckoo hash table ────────────────────────────────────────────────────────
struct CuckooHashTable {
    table1: Vec<Option<i32>>, // first of the two independent tables
    table2: Vec<Option<i32>>, // second table
    size: usize,              // number of slots in EACH table
    count: usize,             // total keys stored (across both tables)
    h1: HashFn,               // hash function dedicated to table1
    h2: HashFn,               // hash function dedicated to table2
    rehash_seed: u64,         // incremented each rehash so we get fresh functions
}

impl CuckooHashTable {
    fn new(size: usize) -> Self {
        CuckooHashTable {
            table1: vec![None; size],
            table2: vec![None; size],
            size,
            count: 0,
            h1: HashFn::new(0),
            h2: HashFn::new(1),
            rehash_seed: 2,
        }
    }

    // ── Public insert ─────────────────────────────────────────────────────────
    // Checks load factor first, then delegates to the internal kick loop.
    // If the kick loop detects a cycle it rehashes and retries — the paper
    // proves the expected number of rehash rounds before success is O(1).
    fn insert(&mut self, key: i32) {
        // Load factor check: total keys / (2 * size) > 0.5  ↔  count > size
        // Only double when load is high; below 50 % we just rehash in place.
        if self.count >= self.size {
            println!("  [load factor high — doubling table size before insert]");
            self.grow_and_rehash();
        }

        // Retry loop: each iteration tries a fresh pair of hash functions.
        loop {
            if self.try_insert(key) {
                self.count += 1;
                return;
            }
            // Cycle detected — pick new hash functions and rebuild.
            println!("  [cycle detected — rehashing with new hash functions]");
            self.rehash_in_place();
            // If rehash itself looped somehow, the outer loop retries.
        }
    }

    // ── Internal kick loop ────────────────────────────────────────────────────
    // Returns true on success, false if a cycle is detected (max kicks hit).
    // Max kicks = floor(log2(n)) as per the original paper.
    fn try_insert(&mut self, key: i32) -> bool {
        let max_kicks = self.max_kicks();
        let mut current = key;

        // Always start by trying table1 first.
        let mut use_table1 = true;

        for kick in 0..=max_kicks {
            let idx = if use_table1 {
                self.h1.apply(current, self.size)
            } else {
                self.h2.apply(current, self.size)
            };

            let slot = if use_table1 {
                &mut self.table1[idx]
            } else {
                &mut self.table2[idx]
            };

            if slot.is_none() {
                // Empty slot found — place the key and we are done.
                *slot = Some(current);
                if kick > 0 {
                    println!("  placed after {} kick(s)", kick);
                }
                return true;
            }

            // Slot occupied: evict the sitting key and put ours in.
            let evicted = slot.unwrap();
            *slot = Some(current);
            current = evicted;

            // Alternate tables on every kick (the cuckoo mechanism).
            use_table1 = !use_table1;
        }

        // Reached max_kicks without finding an empty slot → cycle.
        false
    }

    // ── Rehash in place ───────────────────────────────────────────────────────
    // Picks fresh h₁ / h₂ from the universal family (same size, new seeds).
    // Collects all existing keys and re-inserts them from scratch.
    fn rehash_in_place(&mut self) {
        loop {
            // Advance seed so every rehash round uses different functions.
            self.h1 = HashFn::new(self.rehash_seed);
            self.rehash_seed += 1;
            self.h2 = HashFn::new(self.rehash_seed);
            self.rehash_seed += 1;

            let keys = self.drain_all_keys();
            self.table1 = vec![None; self.size];
            self.table2 = vec![None; self.size];
            self.count = 0;

            let mut ok = true;
            for k in keys {
                if !self.try_insert(k) {
                    ok = false;
                    break;
                }
                self.count += 1;
            }
            if ok {
                return;
            }
            // Extremely rare: new functions also produced a cycle — try again.
            println!("  [rehash round failed — retrying with yet another hash pair]");
        }
    }

    // ── Grow and rehash ───────────────────────────────────────────────────────
    // Called only when load factor exceeds 50 %.
    // Doubles size, picks new functions, rebuilds both tables.
    fn grow_and_rehash(&mut self) {
        self.size *= 2;
        loop {
            self.h1 = HashFn::new(self.rehash_seed);
            self.rehash_seed += 1;
            self.h2 = HashFn::new(self.rehash_seed);
            self.rehash_seed += 1;

            let keys = self.drain_all_keys();
            self.table1 = vec![None; self.size];
            self.table2 = vec![None; self.size];
            self.count = 0;

            let mut ok = true;
            for k in keys {
                if !self.try_insert(k) {
                    ok = false;
                    break;
                }
                self.count += 1;
            }
            if ok {
                return;
            }
            println!("  [grow-rehash cycle — retrying]");
        }
    }

    // ── Lookup ────────────────────────────────────────────────────────────────
    // O(1) worst case: check exactly 2 locations, one per table.
    fn contains(&self, key: i32) -> bool {
        let i1 = self.h1.apply(key, self.size);
        let i2 = self.h2.apply(key, self.size);
        self.table1[i1] == Some(key) || self.table2[i2] == Some(key)
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    // floor(log2(n)) with a safe minimum of 8 so tiny tables still work.
    fn max_kicks(&self) -> usize {
        let n = self.count.max(1);
        let log = usize::BITS as usize - n.leading_zeros() as usize - 1;
        log.max(8)
    }

    // Drain both tables and return all stored keys as a Vec.
    fn drain_all_keys(&mut self) -> Vec<i32> {
        let mut keys = Vec::with_capacity(self.count);
        for slot in self.table1.iter_mut().chain(self.table2.iter_mut()) {
            if let Some(k) = slot.take() {
                keys.push(k);
            }
        }
        keys
    }

    fn print_tables(&self) {
        print!("Table1: [");
        for slot in &self.table1 {
            match slot {
                Some(v) => print!("{:>4}", v),
                None    => print!("   _"),
            }
        }
        println!(" ]");

        print!("Table2: [");
        for slot in &self.table2 {
            match slot {
                Some(v) => print!("{:>4}", v),
                None    => print!("   _"),
            }
        }
        println!(" ]");
    }
}

// ─── Main ─────────────────────────────────────────────────────────────────────
fn main() {
    let mut cuckoo = CuckooHashTable::new(8);
    let keys = vec![10, 25, 37, 99, 24, 23, 11, 1, 12, 55, 67];

    for key in &keys {
        print!("insert({:>3}) → ", key);
        cuckoo.insert(*key);
    }

    println!("\n--- Final state (size per table = {}) ---", cuckoo.size);
    cuckoo.print_tables();

    println!("\n--- Lookup tests ---");
    for key in &[10, 25, 99, 999] {
        println!("contains({:>3}) = {}", key, cuckoo.contains(*key));
    }
}