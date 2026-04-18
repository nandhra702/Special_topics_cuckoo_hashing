use rand::RngExt;
use std::time::Instant;

struct CuckooHashTable{
    table1: Vec<Option<i64>>, //first hash function hashes to this
    table2: Vec<Option<i64>>, //2nd hash function hashes to this
    size: usize, //store the table size. USIZE is unsigned integer

    a1: i64,
    b1: i64,
    a2: i64,
    b2: i64,
    p: i64,

    rehash_count: usize,
    total_kicks: usize,
}


/* we just created the struct. Now unlike cpp, rust doesnt have null. SO we use something
new, AKA Option. SO, its like a vector of options. An option can have 2 values. NONE (empty slot) or Some(any value)
.
So the table would look like : [None, Some(10), None, Some(25), ...]
*/

//fn is the function keyword. impl is keyword for 'implement functionality'
//used for structs, groups functions together under a namespace for that struct

impl CuckooHashTable {
   
    /*This function takes in a size, makes a vector of that size, thats full of nones. Its basically a constructor*/
    
    fn new(size: usize) -> Self {
    let mut rng = rand::rng();  // ✅ define first

    CuckooHashTable {
        table1: vec![None; size],
        table2: vec![None; size],
        size,

        a1: rng.random_range(1..1_000_000_007),
        b1: rng.random_range(0..1_000_000_007),
        a2: rng.random_range(1..1_000_000_007),
        b2: rng.random_range(0..1_000_000_007),
        p: 1_000_000_007,

        rehash_count: 0,
        total_kicks: 0,
    }
}

    //THE SAID HASH FUNCTIONS UNIVERSAL HASHING : h(x)=((a⋅x+b)modp)modm
    fn hash1(&self, key: i64) -> usize {
        (((self.a1 * key + self.b1) % self.p) as usize) % self.size
    }

    fn hash2(&self, key: i64) -> usize {
        (((self.a2 * key + self.b2) % self.p) as usize) % self.size
    }

    //FUNCTION TO REGENERATE THE NEW HASHES DIRECTLY USED BY REHASHER
    fn regenerate_hashes(&mut self) {
        let mut rng = rand::rng();
        self.a1 = rng.random_range(1..self.p);
        self.b1 = rng.random_range(0..self.p);
        self.a2 = rng.random_range(1..self.p);
        self.b2 = rng.random_range(0..self.p);
    }

    //ON REHASHING, double the size of the table
    fn rehash(&mut self) {
        println!("Rehash #{}, size = {}", self.rehash_count, self.size);
        if self.rehash_count > 20 {
        panic!("Too many rehashes — aborting to prevent OOM");
        }
        self.rehash_count += 1;

        let old_table1 = self.table1.clone();
        let old_table2 = self.table2.clone();

        // resize 
        if self.rehash_count % 2 == 0 {
        self.size *= 2;
        }

        self.table1 = vec![None; self.size];
        self.table2 = vec![None; self.size];

        self.regenerate_hashes();

        for entry in old_table1.into_iter().chain(old_table2.into_iter()) {
            if let Some(key) = entry {
                self.insert(key);
            }
        }
    }

    //STAR INSERT FUNCTION
    fn insert(&mut self, key: i64) -> bool {

        if self.contains(key) {
           return true;
        }

        let mut current = key;
        let mut table_id = 1;

        let max_kicks = 500; //hardcode

        for _ in 0..max_kicks {
            if table_id == 1 {
                let i = self.hash1(current);

                if self.table1[i].is_none() {
                    self.table1[i] = Some(current);
                    return true;
                }

                // kick
                let displaced = self.table1[i].unwrap();
                self.table1[i] = Some(current);
                current = displaced;
                table_id = 2;
                self.total_kicks += 1;

            } else {
                let i = self.hash2(current);

                if self.table2[i].is_none() {
                    self.table2[i] = Some(current);
                    return true;
                }

                // kick
                let displaced = self.table2[i].unwrap();
                self.table2[i] = Some(current);
                current = displaced;
                table_id = 1;
                self.total_kicks += 1;
            }
        }

        // cycle detected
        self.rehash();
        return self.insert(current)
    }

    fn contains(&self, key: i64) -> bool {
        let i1 = self.hash1(key);
        if self.table1[i1] == Some(key) {
            return true;
        }

        let i2 = self.hash2(key);
        if self.table2[i2] == Some(key) {
            return true;
        }

        false
    }
}

/* ================= BENCHMARKING ================= */
/*
fn generate_dataset(n: usize, range: i64) -> Vec<i64> {
    let mut rng = rand::rng();
    let mut data = Vec::with_capacity(n);

    for _ in 0..n {
        data.push(rng.random_range(1..range));
    }

    data
}*/

use std::collections::HashSet;

fn generate_dataset(n: usize, range: i64) -> Vec<i64> {
    let mut rng = rand::rng();
    let mut set = HashSet::new();

    while set.len() < n {
        set.insert(rng.random_range(1..range));
    }

    set.into_iter().collect()
}

fn benchmark_insert(data: &Vec<i64>) -> CuckooHashTable {
    let mut table = CuckooHashTable::new(data.len() * 2);

    let start = Instant::now();

    for &key in data {
        table.insert(key);
    }

    let duration = start.elapsed();

    println!("Insert time: {:?}", duration);
    println!("Rehashes: {}", table.rehash_count);
    println!("Total kicks: {}", table.total_kicks);

    table
}

fn benchmark_lookup_found(table: &CuckooHashTable, data: &Vec<i64>) {
    let start = Instant::now();

    for &key in data {
        table.contains(key);
    }

    let duration = start.elapsed();
    println!("Lookup (found): {:?}", duration);
}

fn benchmark_lookup_not_found(table: &CuckooHashTable, data: &Vec<i64>) {
    let start = Instant::now();

    for &key in data {
        table.contains(key + 1_000_000_000);
    }

    let duration = start.elapsed();
    println!("Lookup (not found): {:?}", duration);
}

fn load_factor_experiment() {
    let loads = [0.5, 0.7, 0.85, 0.9];

    println!("\n--- Load Factor Experiment ---");

    for &load in &loads {
        let n = 50_000;
        let capacity = (n as f64 /(2.0*load)) as usize;     //AS WE have 2 tables, so twice the capacity right

        let data = generate_dataset(20_000, 1_000_000);

        let mut table = CuckooHashTable::new(capacity);

        let start = Instant::now();

        for &key in &data {
            table.insert(key);
        }

        let duration = start.elapsed();

        println!(
            "Load {:.2} → time {:?}, rehashes {}, kicks {}",
            load, duration, table.rehash_count, table.total_kicks
        );
    }
}

fn memory_usage(table: &CuckooHashTable) {
    let total_slots = 2 * table.size;
    let bytes = total_slots * std::mem::size_of::<Option<i64>>();

    println!("Total slots: {}", total_slots);
    println!("Approx memory (bytes): {}", bytes);
}

/* ================= MAIN ================= */

fn main() {
    println!("--- Cuckoo Hashing Benchmark ---");

    let dataset = generate_dataset(100_000, 1_000_000);

    let table = benchmark_insert(&dataset);

    benchmark_lookup_found(&table, &dataset);
    benchmark_lookup_not_found(&table, &dataset);

    memory_usage(&table);

    load_factor_experiment();
}