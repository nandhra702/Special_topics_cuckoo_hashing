use rand::RngExt;
use std::time::Instant;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/* ================= STATS ================= */

#[derive(Default, Clone)]
struct Stats {
    capacity: usize,
    size: usize,
    load_factor: f64,
    empty_cells: usize,
    tombstones: usize,

    insert_calls: u64,
    find_calls: u64,

    total_probes_insert: u64,
    total_probes_find: u64,
}

/* ================= CUCKOO ================= */

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

    // stats
    insert_calls: u64,
    find_calls: u64,
    successful_finds: u64,
    failed_finds: u64,
    total_probes_insert: u64,
    total_probes_find: u64,
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

            insert_calls: 0,
            find_calls: 0,
            successful_finds: 0,
            failed_finds: 0,
            total_probes_insert: 0,
            total_probes_find: 0,
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
        self.insert_calls += 1;

        if self.contains(key) {
            return true;
        }

        let mut current = key;
        let mut table_id = 1;
        let max_kicks = 500;

        let mut probes = 0;

        for _ in 0..max_kicks {
            probes += 1;

            if table_id == 1 {
                let i = self.hash1(current);

                if self.table1[i].is_none() {
                    self.table1[i] = Some(current);
                    self.total_probes_insert += probes;
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
                    self.total_probes_insert += probes;
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

    fn contains(&mut self, key: i64) -> bool {
        self.find_calls += 1;

        let mut probes = 1;
        let i1 = self.hash1(key);

        if self.table1[i1] == Some(key) {
            self.total_probes_find += probes;
            self.successful_finds += 1;
            return true;
        }

        probes += 1;
        let i2 = self.hash2(key);

        if self.table2[i2] == Some(key) {
            self.total_probes_find += probes;
            self.successful_finds += 1;
            return true;
        }

        self.total_probes_find += probes;
        self.failed_finds += 1;
        false
    }

    fn size(&self) -> usize {
        self.table1.iter().filter(|x| x.is_some()).count()
            + self.table2.iter().filter(|x| x.is_some()).count()
    }

    fn capacity(&self) -> usize {
        self.size * 2
    }

    fn load_factor(&self) -> f64 {
        self.size() as f64 / self.capacity() as f64
    }

    fn empty_cells(&self) -> usize {
        self.capacity() - self.size()
    }
}

/* ================= DATASET HELPERS ================= */

fn read_lines(path: &str, limit: usize) -> Vec<String> {
    let file = File::open(path).expect("Cannot open file");
    let reader = BufReader::new(file);

    reader.lines().take(limit).map(|l| l.unwrap()).collect()
}

fn string_to_i64(s: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as i64
}

fn ip_to_i64(ip: &str) -> i64 {
    let parts: Vec<u8> = ip.split('.').map(|x| x.parse::<u8>().unwrap()).collect();

    ((parts[0] as i64) << 24)
        | ((parts[1] as i64) << 16)
        | ((parts[2] as i64) << 8)
        | (parts[3] as i64)
}

/* ================= DATASET LOADERS ================= */

fn load_google_words(path: &str, limit: usize) -> Vec<i64> {
    read_lines(path, limit).into_iter().map(|s| string_to_i64(&s)).collect()
}

fn load_wiki_words(path: &str, limit: usize) -> Vec<i64> {
    read_lines(path, limit).into_iter().map(|s| string_to_i64(&s)).collect()
}

fn load_osm_ids(path: &str, limit: usize) -> Vec<i64> {
    read_lines(path, limit).into_iter().map(|s| s.parse::<i64>().unwrap()).collect()
}

/* ================= BENCHMARK ================= */

fn run_dataset(name: &str, data: Vec<i64>) {
    println!("\n=== {} ===", name);

    let load_factors = [0.5, 0.7, 0.8, 0.9, 0.95];

    let mut file = File::create(format!("{}_cuckoo.csv", name)).unwrap();

    writeln!(
        file,
        "dataset,load_factor,capacity,size,empty_cells,tombstones,\
        avg_probes_insert,avg_probes_find_hit,avg_probes_find_miss,\
        insert_ns_per_op,find_ns_per_op"
    ).unwrap();

    for &lf in &load_factors {
        let n = (data.len() as f64 * lf) as usize;
        let subset = &data[..n];

        let mut table = CuckooHashTable::new(n * 2);

        // INSERT
        let start = Instant::now();
        for &key in subset {
            table.insert(key);
        }
        let insert_ns = start.elapsed().as_nanos() as f64 / n as f64;

        // FIND
        let start = Instant::now();

        for &key in subset {
            table.contains(key);
        }

        for &key in subset {
            table.contains(key + 1_000_000_000);
        }

        let find_ns = start.elapsed().as_nanos() as f64 / (2 * n) as f64;

        let avg_insert = table.total_probes_insert as f64 / table.insert_calls as f64;
        let avg_find = table.total_probes_find as f64 / table.find_calls as f64;

        writeln!(
            file,
            "{},{:.2},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2}",
            name,
            lf,
            table.capacity(),
            table.size(),
            table.empty_cells(),
            0,
            avg_insert,
            avg_find,
            avg_find,
            insert_ns,
            find_ns
        ).unwrap();

        println!("LF {:.2} → insert {:.2}ns, find {:.2}ns", lf, insert_ns, find_ns);
    }
}

/* ================= MAIN ================= */

fn main() {
    println!("--- Cuckoo Hashing Benchmark ---");

    let google = load_google_words("data/google-10000-english.txt", 10_000);
    run_dataset("google", google);

    let wiki = load_wiki_words("data/enwiki-latest-all-titles-in-ns0", 50_000);
    run_dataset("wiki", wiki);

    let osm = load_osm_ids("data/osm_ids.txt", 100_000);
    run_dataset("osm", osm);
}