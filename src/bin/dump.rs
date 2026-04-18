use rand::RngExt;

struct CuckooHashTable{
    table1: Vec<Option<i64>>, //first hash function hashes to this
    table2: Vec<Option<i64>>, //2nd hash function hashes to this
    size: usize, //store the table size. USIZE is unsigned integer

    a1: i64,
    b1: i64,
    a2: i64,
    b2: i64,
    p: i64,
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
        CuckooHashTable {
            table1: vec![None; size],
            table2: vec![None; size],
            size,

            a1: 3,
            b1: 7,
            a2: 5,
            b2: 11,
            p: 1_000_000_007,
        }
    }

    //THE SAID HASH FUNCTIONS UNIVERSAL HASHING : h(x)=((a⋅x+b)modp)modm
    fn hash1(&self, key: i64) -> usize {
    (((self.a1 * key + self.b1) % self.p) as usize) % self.size
    }

    fn hash2(&self, key: i64) -> usize {
        (((self.a2 * key + self.b2) % self.p) as usize) % self.size
    }

    //BASIC INSERTION INTO HASHTABLE. FIRST COMPUTE H1, check in table. Then if not empty, check in slot 2
    /*fn insert_simple(&mut self, key: i64) -> bool {
    let i1 = self.hash1(key);

    if self.table[i1].is_none() {
        self.table[i1] = Some(key);
        return true;
    }

    let i2 = self.hash2(key);

    if self.table[i2].is_none() {
        self.table[i2] = Some(key);
        return true;
    }

    false
    }*/



    //FUNCTION TO REGENERATE THE NEW HASHES DIRECTLY USED BY REHASHER
    fn regenerate_hashes(&mut self) {
    let mut rng = rand::rng();
    self.a1 = rng.random_range(1..self.p);   // gen_range → random_range
    self.b1 = rng.random_range(0..self.p);
    self.a2 = rng.random_range(1..self.p);
    self.b2 = rng.random_range(0..self.p);
}

    //ON REHASHING, double the size of the table

    fn rehash(&mut self) {
        let old_table1 = self.table1.clone();
        let old_table2 = self.table2.clone();

        // resize 
        self.size *= 2;

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
            let mut current = key;
            let mut table_id = 1;

            let max_kicks = self.size;

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
                }
            }
        
            // cycle detected
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




fn main() {
    let mut cuckoo = CuckooHashTable::new(10);
    let mut rng = rand::rng();              // ← rand 0.9 syntax
    let n = 20;
    let mut inserted_keys = Vec::new();

    println!("--- Inserting Keys ---");
    for _ in 0..n {
        let key = rng.random_range(1..1000);  // ← rand 0.9 syntax
        let success = cuckoo.insert(key);
        println!("Insert {:>4} → {}", key, if success { "OK" } else { "FAILED" });
        if success {
            inserted_keys.push(key);
        }
    }

    println!("\n--- Final Tables ---");
    println!("Table 1:");
    for (i, val) in cuckoo.table1.iter().enumerate() {
        println!("Index {:>2}: {:?}", i, val);
    }
    println!("\nTable 2:");
    for (i, val) in cuckoo.table2.iter().enumerate() {
        println!("Index {:>2}: {:?}", i, val);
    }

    println!("\n--- Verification ---");
    for key in &inserted_keys {
    if !cuckoo.contains(*key) {
        println!("ERROR: Key {} not found!", key);
    }
    }

    let total_slots = 2 * cuckoo.size;
    let filled = cuckoo.table1.iter().filter(|x| x.is_some()).count()
        + cuckoo.table2.iter().filter(|x| x.is_some()).count();

    println!("\n--- Stats ---");
    println!("Inserted keys: {}", inserted_keys.len());
    println!("Total slots: {}", total_slots);
    println!("Load factor: {:.3}", filled as f64 / total_slots as f64);
}