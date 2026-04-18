use rand::RngExt;
use std::time::Instant;
use std::fs::File;
use std::io::{BufRead, BufReader};

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

struct CuckooHashTable {
    table1: Vec<Option<i64>>,
    table2: Vec<Option<i64>>,
    size: usize,

    a1: i64,
    b1: i64,
    a2: i64,
    b2: i64,
    p: i64,

    rehash_count: usize,
    total_kicks: usize,
}

impl CuckooHashTable {

    fn new(size: usize) -> Self {
        let mut rng = rand::rng();

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

    fn hash1(&self, key: i64) -> usize {
        (((self.a1 * key + self.b1) % self.p) as usize) % self.size
    }

    fn hash2(&self, key: i64) -> usize {
        (((self.a2 * key + self.b2) % self.p) as usize) % self.size
    }

    fn regenerate_hashes(&mut self) {
        let mut rng = rand::rng();
        self.a1 = rng.random_range(1..self.p);
        self.b1 = rng.random_range(0..self.p);
        self.a2 = rng.random_range(1..self.p);
        self.b2 = rng.random_range(0..self.p);
    }

    fn rehash(&mut self) {
        println!("Rehash #{}, size = {}", self.rehash_count, self.size);

        if self.rehash_count > 20 {
            panic!("Too many rehashes — aborting");
        }

        self.rehash_count += 1;

        let old_table1 = self.table1.clone();
        let old_table2 = self.table2.clone();

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

    fn insert(&mut self, key: i64) -> bool {
        if self.contains(key) {
            return true;
        }

        let mut current = key;
        let mut table_id = 1;
        let max_kicks = 500;

        for _ in 0..max_kicks {
            if table_id == 1 {
                let i = self.hash1(current);

                if self.table1[i].is_none() {
                    self.table1[i] = Some(current);
                    return true;
                }

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

                let displaced = self.table2[i].unwrap();
                self.table2[i] = Some(current);
                current = displaced;
                table_id = 1;
                self.total_kicks += 1;
            }
        }

        self.rehash();
        self.insert(current)
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

/* ================= DATASET HELPERS ================= */

fn read_lines(path: &str, limit: usize) -> Vec<String> {
    let file = File::open(path).expect("Cannot open file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .take(limit)
        .map(|l| l.unwrap())
        .collect()
}

fn string_to_i64(s: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as i64
}

fn ip_to_i64(ip: &str) -> i64 {
    let parts: Vec<u8> = ip
        .split('.')
        .map(|x| x.parse::<u8>().unwrap())
        .collect();

    ((parts[0] as i64) << 24)
        | ((parts[1] as i64) << 16)
        | ((parts[2] as i64) << 8)
        | (parts[3] as i64)
}

/* ================= DATASET LOADERS ================= */

fn load_google_words(path: &str, limit: usize) -> Vec<i64> {
    read_lines(path, limit)
        .into_iter()
        .map(|s| string_to_i64(&s))
        .collect()
}

fn load_wiki_words(path: &str, limit: usize) -> Vec<i64> {
    read_lines(path, limit)
        .into_iter()
        .map(|s| string_to_i64(&s))
        .collect()
}

fn load_ip_dataset(path: &str, limit: usize) -> Vec<i64> {
    read_lines(path, limit)
        .into_iter()
        .map(|ip| ip_to_i64(&ip))
        .collect()
}

fn load_osm_ids(path: &str, limit: usize) -> Vec<i64> {
    read_lines(path, limit)
        .into_iter()
        .map(|s| s.parse::<i64>().unwrap())
        .collect()
}

/* ================= BENCHMARK ================= */

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

    println!("Lookup (found): {:?}", start.elapsed());
}

fn benchmark_lookup_not_found(table: &CuckooHashTable, data: &Vec<i64>) {
    let start = Instant::now();

    for &key in data {
        table.contains(key + 1_000_000_000);
    }

    println!("Lookup (not found): {:?}", start.elapsed());
}

fn memory_usage(table: &CuckooHashTable) {
    let total_slots = 2 * table.size;
    let bytes = total_slots * std::mem::size_of::<Option<i64>>();

    println!("Total slots: {}", total_slots);
    println!("Approx memory (bytes): {}", bytes);
}

/* ================= RUNNER ================= */

fn run_dataset(name: &str, data: Vec<i64>) {
    println!("\n=== {} ===", name);

    let table = benchmark_insert(&data);
    benchmark_lookup_found(&table, &data);
    benchmark_lookup_not_found(&table, &data);
    memory_usage(&table);
}

/* ================= MAIN ================= */

fn main() {
    println!("--- Real Dataset Benchmark ---");

    // 🔹 Google words (START HERE)
    let google = load_google_words("data/google-10000-english.txt", 10_000);
    run_dataset("Google Words", google);

    // 🔹 Wikipedia
    let wiki = load_wiki_words("data/enwiki-latest-all-titles-in-ns0", 50_000); 
    run_dataset("Wikipedia", wiki);

    // 🔹 IP dataset
    //] let ip = load_ip_dataset("data/ip.txt", 50_000);
    // run_dataset("IP Dataset", ip);

    // 🔹 OSM IDs
    let osm = load_osm_ids("data/osm_ids.txt", 100_000);
    run_dataset("OSM IDs", osm);
}