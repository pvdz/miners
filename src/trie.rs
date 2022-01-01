// Send help (:

// A Trie ("try") is a data model where strings that partially overlap share the same branching path.
// It's quite efficient at encoding a dictionary, for example. The vertex at the end of a string has
// the value of that string. The key does not need to be a string, per say, it's just a good example.

// In our game we'll use it to encode the visiting order, which is our pseudo encoding scheme.

// The implementation is basically a list of nodes where each field points to another node in the
// list. Each node can also have a value, which tells you whether it was known or not. Zeroes
// indicate non-existence.

// The encoding scheme takes the x,y coords and serializes them in a string, appending a dash to
// every number. This way a unique hash is formed with a string that only contains the digits
// 0 through 9 and a dash. We use these 11 characters to form the Trie.

// This makes a minimal path take up: `0-0-` = 4 steps, meaning 4x12xi32=192 bytes.
// Will have to see whether serializing that is feasible for distribution or not :)

use std::fmt;
// use super::icons::*;

const TRIE_PANIC: i8 = 0;
const TRIE_OVERWRITE: i8 = 1;
const TRIE_IGNORE: i8 = 2;

// A node has 11 positions as a jump table and one "value" position.
// The value position is 0 if "not found" (etc), and non-zero if known (TBD)
pub type TrieNode = [usize; 12];

pub struct Trie {
    nodes: Vec<TrieNode>,
}

impl Trie {
    pub fn new() -> Trie {
        return Trie {
            nodes: vec!(
                [0 as usize; 12],
            ),
        };
    }

    pub fn read(&mut self, trail: &str) -> usize {
        return self.walk(trail, false, 0);
    }

    pub fn write(&mut self, trail: &str, value: usize) {
        self.walk(trail, true, value);
    }

    // A walker that can optionally update the trie if the value is not found
    fn walk(&mut self, trail: &str, write: bool, value: usize) -> usize {

        // let mut t: [u32; 12] = trie[0];
        let mut t: TrieNode = self.nodes[0];
        let mut ti: usize = 0;
        let mut step: u32 = 0;
        let mut writing: bool = false;
        let mut len: usize = self.nodes.len();

        for (i, c) in trail.bytes().enumerate() {
            if !writing {
                let next_ti: usize = match c {
                    48 => t[0],
                    49 => t[1],
                    50 => t[2],
                    51 => t[3],
                    52 => t[4],
                    53 => t[5],
                    54 => t[6],
                    55 => t[7],
                    56 => t[8],
                    57 => t[9],
                    45 => t[10],
                    _x => panic!("Encoded string should only contain digits and dashes, got {}", _x),
                };

                println!("Step: {}, path index: {}, c: {}, node index: {}, next node index: {}", step, i, c, ti, next_ti);
                step = step + 1;

                if next_ti != 0 {
                    ti = next_ti;
                    t = self.nodes[ti];
                    continue;
                }

                if !write {
                    // "Not found"
                    return 0;
                }

                writing = true
            }

            if writing {
                println!("Writing {} to index {} on node {}", len, c, ti);
                match c {
                    48 => self.nodes[ti][0] = len,
                    49 => self.nodes[ti][1] = len,
                    50 => self.nodes[ti][2] = len,
                    51 => self.nodes[ti][3] = len,
                    52 => self.nodes[ti][4] = len,
                    53 => self.nodes[ti][5] = len,
                    54 => self.nodes[ti][6] = len,
                    55 => self.nodes[ti][7] = len,
                    56 => self.nodes[ti][8] = len,
                    57 => self.nodes[ti][9] = len,
                    45 => self.nodes[ti][10] = len,
                    _x => panic!("Encoded string should only contain digits and dashes, got {}", _x),
                };
                ti = len;
                t = [0; 12];
                self.nodes.push(t);
                len = len + 1;
            }
        }

        if write {
            if t[11] != 0 {
                panic!("Not expecting to clobber value.");
            }

            println!("Now writing {} to node {}", value, ti);
            self.nodes[ti][11] = value;
            return value;
        }

        return t[11];
    }

    pub fn path_to_trail(&self, path: Vec<i32>) -> String {
        let mut hash = String::new();

        for num in path {
            hash.push_str(&num.to_string());
            hash.push_str("-");
        }

        println!("{}", hash);

        return hash;
    }
}

impl fmt::Display for Trie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Trie[ {} :\n", self.nodes.len());
        for node in self.nodes.iter() {
            write!(f, "  {:?},\n", node);
        }
        write!(f, "]")
    }
}
