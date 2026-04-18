struct CuckooHashTable{
    table1: Vec<Option<i32>>, //first hash function hashes to this
    table2: Vec<Option<i32>>, //2nd hash function hashes to this
    size: usize, //store the table size. USIZE is unsigned integer

    a1: i32,
    b1: i32,
    a2: i32,
    b2: i32,
    p: i32,
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
    
    impl CuckooHashTable {
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
}

    //THE SAID HASH FUNCTIONS
    fn hash1(&self, key: i32) -> usize {
        (key as usize) % self.size
    }

    fn hash2(&self, key: i32) -> usize {
        ((key as usize) / self.size) % self.size
    }

    //BASIC INSERTION INTO HASHTABLE. FIRST COMPUTE H1, check in table. Then if not empty, check in slot 2
    fn insert_simple(&mut self, key: i32) -> bool {
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
    }

    //STAR INSERT FUNCTION
    fn insert(&mut self, key: i32) -> bool {
    let mut current_key = key;
    let mut index = self.hash1(current_key);

    let max_kicks = self.size; // simple limit, keep cycling till count of kicking out reaches the table size

    for _ in 0..max_kicks {
        if self.table[index].is_none() {
            self.table[index] = Some(current_key);
            return true;
        }

        // Kick out existing key
        let displaced = self.table[index].unwrap(); //extracts value from SOME(VALUE)
        self.table[index] = Some(current_key);

        current_key = displaced;

        // Switch position
        if index == self.hash1(current_key) { //if we came from hash1 function, go to hash2 function
            index = self.hash2(current_key);
        } else {
            index = self.hash1(current_key);
        }
    }

    // Failed (cycle detected)
    self.rehash();
    self.insert(current_key)
    }


    //ON REHASHING, double the size of the table

        fn rehash(&mut self) {
        let old_table = self.table.clone();

        self.size *= 2;
        self.table = vec![None; self.size];

        for entry in old_table {
            if let Some(key) = entry {
                self.insert(key);
            }
        }
        }





}



fn main() {
    let mut cuckoo = CuckooHashTable::new(10); //cause we are editing the table, hence mutable

    let keys = vec![10, 25, 37, 99, 15, 20, 30, 45, 60, 75];

    for key in keys {
        let result = cuckoo.insert(key);
        println!("Insert {} -> {}", key, result);
    }

    println!("Final Table: {:?}", cuckoo.table); //why do I have to weite {:?} ? TO BE CHECKED !!!!!!!!!!!!!!!!!!
}