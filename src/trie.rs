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

// With octal ints:
// Trie is in Octal mode. It has 1216598 nodes. It contains  76 unique paths out of  100 total. Each node takes up 4x10 bytes so naive encoding totals  9732792 bytes
// Trie is in Octal mode. It has 3035522 nodes. It contains 324 unique paths out of 1000 total. Each node takes up 4x10 bytes so naive encoding totals 24284184 bytes
// Trie is in Octal mode. It has 4730023 nodes. It contains 507 unique paths out of 2720 total. Each node takes up 4x10 bytes so naive encoding totals 37840192 bytes
// Trie is in Octal mode. It has 4945166 nodes. It contains 554 unique paths out of 3510 total. Each node takes up 4x10 bytes so naive encoding totals 39561336 bytes

// With decimal ints:
// Trie is in Decimal mode. It has  233915 nodes. It contains  62 unique paths out of    90 total. Each node takes up 4x12 bytes so naive encoding totals  1871330 bytes
// Trie is in Decimal mode. It has  342984 nodes. It contains 105 unique paths out of   260 total. Each node takes up 4x12 bytes so naive encoding totals  2743882 bytes
// Trie is in Decimal mode. It has  422031 nodes. It contains 146 unique paths out of   440 total. Each node takes up 4x12 bytes so naive encoding totals  3376258 bytes
// Trie is in Decimal mode. It has  543394 nodes. It contains 193 unique paths out of  1000 total. Each node takes up 4x12 bytes so naive encoding totals  4347162 bytes
// Trie is in Decimal mode. It has  639917 nodes. It contains 223 unique paths out of  1680 total. Each node takes up 4x12 bytes so naive encoding totals  5119346 bytes
// Trie is in Decimal mode. It has 1158823 nodes. It contains 415 unique paths out of  4040 total. Each node takes up 4x12 bytes so naive encoding totals  9270594 bytes
// Trie is in Decimal mode. It has 1301701 nodes. It contains 467 unique paths out of  6210 total. Each node takes up 4x12 bytes so naive encoding totals 10413618 bytes
// Trie is in Decimal mode. It has 1344687 nodes. It contains 488 unique paths out of  7370 total. Each node takes up 4x12 bytes so naive encoding totals 10757506 bytes
// Trie is in Decimal mode. It has 1411237 nodes. It contains 510 unique paths out of  8610 total. Each node takes up 4x12 bytes so naive encoding totals 11289906 bytes
// Trie is in Decimal mode. It has 1438680 nodes. It contains 534 unique paths out of 10000 total. Each node takes up 4x12 bytes so naive encoding totals 11509450 bytes

// With hex:
// Trie is in Hex mode.     It has  683226 nodes. It contains 108 unique paths out of   160 total. Each node takes up 4x18 bytes so naive encoding totals  5465824 bytes
// Trie is in Hex mode.     It has 1407459 nodes. It contains 263 unique paths out of   610 total. Each node takes up 4x18 bytes so naive encoding totals 11259688 bytes
// Trie is in Hex mode.     It has 1492411 nodes. It contains 307 unique paths out of  1000 total. Each node takes up 4x18 bytes so naive encoding totals 11939304 bytes
// Trie is in Hex mode.     It has 1559908 nodes. It contains 340 unique paths out of  1560 total. Each node takes up 4x18 bytes so naive encoding totals 12479280 bytes
// Trie is in Hex mode.     It has 1740405 nodes. It contains 402 unique paths out of  3300 total. Each node takes up 4x18 bytes so naive encoding totals 13923256 bytes
// Trie is in Hex mode.     It has 1904938 nodes. It contains 422 unique paths out of  4080 total. Each node takes up 4x18 bytes so naive encoding totals 15239520 bytes
// Trie is in Hex mode.     It has 2061122 nodes. It contains 461 unique paths out of  5970 total. Each node takes up 4x18 bytes so naive encoding totals 16488992 bytes
// Trie is in Hex mode.     It has 2100944 nodes. It contains 482 unique paths out of  7420 total. Each node takes up 4x18 bytes so naive encoding totals 16807568 bytes
// Trie is in Hex mode.     It has 2128471 nodes. It contains 491 unique paths out of  8230 total. Each node takes up 4x18 bytes so naive encoding totals 17027784 bytes
// Trie is in Hex mode.     It has 2147497 nodes. It contains 503 unique paths out of  8930 total. Each node takes up 4x18 bytes so naive encoding totals 17179992 bytes
// Trie is in Hex mode.     It has 2175903 nodes. It contains 514 unique paths out of 10000 total. Each node takes up 4x18 bytes so naive encoding totals 17407240 bytes

// Trie mode: Binary     It has    2076220 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      29604) out of      18240 total. Each node stores   2+2 x i32 so naive encoding totals (  2+2)*4*2076220   =   16222 kb
// Trie mode: B3         It has    1546828 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      22068) out of      18240 total. Each node stores   3+2 x i32 so naive encoding totals (  3+2)*4*1546828   =   12087 kb
// Trie mode: B4         It has    1362602 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      19372) out of      18240 total. Each node stores   4+2 x i32 so naive encoding totals (  4+2)*4*1362602   =   10649 kb
// Trie mode: B5         It has    1264003 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      18028) out of      18240 total. Each node stores   5+2 x i32 so naive encoding totals (  5+2)*4*1264003   =    9880 kb
// Trie mode: B6         It has    1200845 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      17061) out of      18240 total. Each node stores   6+2 x i32 so naive encoding totals (  6+2)*4*1200845   =    9387 kb
// Trie mode: B7         It has    1153015 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      16360) out of      18240 total. Each node stores   7+2 x i32 so naive encoding totals (  7+2)*4*1153015   =    9014 kb
// Trie mode: Octal      It has    2393366 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      37631) out of      18240 total. Each node stores   8+2 x i32 so naive encoding totals (  8+2)*4*2393366   =   18706 kb
// Trie mode: Decimal    It has    1072570 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      15319) out of      18240 total. Each node stores  10+2 x i32 so naive encoding totals ( 10+2)*4*1072570   =    8389 kb
// Trie mode: Hex        It has    2098388 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      32649) out of      18240 total. Each node stores  16+2 x i32 so naive encoding totals ( 16+2)*4*2098388   =   16409 kb
// Trie mode: Alpha      It has     963074 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      13799) out of      18240 total. Each node stores  26+2 x i32 so naive encoding totals ( 26+2)*4*963074    =    7550 kb
// Trie mode: Alnum      It has     914800 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      13193) out of      18240 total. Each node stores  36+2 x i32 so naive encoding totals ( 36+2)*4*914800    =    7182 kb
// Trie mode: AlUp       It has     846872 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      12160) out of      18240 total. Each node stores  62+2 x i32 so naive encoding totals ( 62+2)*4*846872    =    6678 kb
// Trie mode: B125       It has     808493 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:      11485) out of      18240 total. Each node stores 125+2 x i32 so naive encoding totals (125+2)*4*808493    =    6441 kb
// Trie mode: BYTE       It has    1294089 nodes. It contains        350 unique paths (avg miner steps:       1611, avg trie path len:       2119) out of      18240 total. Each node stores 256+1 x i32 so naive encoding totals (256+1)*4*1294089   = 1299144 kb
// Binary tree mode      It has        350 nodes. It contains        350 unique paths (avg miner steps:       1611, avg search len   :       2119) out of      18240 total. Each node stores (1+2*4*len) x i32 so naive totals      i32*4*2*350*2119  =    5794 kb

use std::fmt;
// use super::icons::*;

const TRIE_PANIC: i8 = 0;
const TRIE_OVERWRITE: i8 = 1;
const TRIE_IGNORE: i8 = 2;

#[derive(Debug)]
pub enum Base {
    Octal = 8,
    Decimal = 10,
    Hex = 16,
}
const BASE: Base = Base::Octal;

// A node has 11 positions as a jump table (10 digits and a dash) and one "value" position.
// The value position is 0 if "not found" (etc), and non-zero if known (TBD)
// In hex mode, there are six more positions (a-f)
pub type TrieNode = [usize; BASE as usize + 2];

pub struct Trie {
    pub nodes: Vec<TrieNode>,
}

impl Trie {
    pub fn new() -> Trie {
        return Trie {
            nodes: vec!(
                [0 as usize; BASE as usize + 2],
            ),
        };
    }

    pub fn read(&mut self, trail: &str, trace: bool) -> usize {
        return self.walk(trail, false, 0, false, trace);
    }

    pub fn write(&mut self, trail: &str, value: usize, clobber: bool, trace: bool) {
        self.walk(trail, true, value, clobber, trace);
    }

    // A walker that can optionally update the trie if the value is not found
    fn walk(&mut self, trail: &str, write: bool, value: usize, clobber: bool, trace: bool) -> usize {

        // let mut t: [u32; 12] = trie[0];
        let mut t: TrieNode = self.nodes[0];
        let mut ti: usize = 0;
        let mut step: u32 = 0;
        let mut writing: bool = false;
        let mut len: usize = self.nodes.len();

        for (i, c) in trail.bytes().enumerate() {
            if !writing {
                let next_ti: usize = t[match c {
                    // - (dash)
                    45 => 1,

                    // 0-9
                    48 => 2,
                    49 => 3,
                    50 => 4,
                    51 => 5,
                    52 => 6,
                    53 => 7,
                    54 => 8,
                    55 => 9,
                    56 => 10,
                    57 => 11,

                    // a-f
                    97 =>  12,
                    98 =>  13,
                    99 =>  14,
                    100 => 15,
                    101 => 16,
                    102 => 17,

                    _x => panic!("Encoded string should only contain digits and dashes, got {}", _x),
                }];

                if trace { println!("Step: {}, path index: {}, c: {}, node index: {}, next node index: {}", step, i, c, ti, next_ti); }
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
                if trace { println!("Writing {} to index {} on node {}", len, c, ti); }
                self.nodes[ti][match c {
                    // Note: index 0 is the value
                    // - (dash)
                    45 => 1,
                    // 0-9
                    48 => 2,
                    49 => 3,
                    50 => 4,
                    51 => 5,
                    52 => 6,
                    53 => 7,
                    54 => 8,
                    55 => 9,
                    56 => 10,
                    57 => 11,
                    // a-f
                    97 =>  12,
                    98 =>  13,
                    99 =>  14,
                    100 => 15,
                    101 => 16,
                    102 => 17,
                    _x => panic!("Encoded string should only contain digits and dashes, got {}", _x),
                }] = len;
                ti = len;
                t = [0; BASE as usize + 2];
                self.nodes.push(t);
                len = len + 1;
            }
        }

        if write {
            if !clobber && t[0] != 0 {
                panic!("Not expecting to clobber value.");
            }

            if trace { println!("Now writing {} to node {}", value, ti); }
            self.nodes[ti][0] = value;
            return value;
        }

        return t[0];
    }

    pub fn path_to_trail(&self, path: &Vec<(i32, i32)>, trace: bool) -> String {
        let mut hash = String::new();

        for (x, y) in path {
            match BASE {
                Base::Octal => hash.push_str(&format!("{:o}", x)),
                Base::Decimal => hash.push_str(&x.to_string()),
                Base::Hex => hash.push_str(&format!("{:x}", x)),
            }
            hash.push_str("-");
            match BASE {
                Base::Octal => hash.push_str(&format!("{:o}", y)),
                Base::Decimal => hash.push_str(&y.to_string()),
                Base::Hex => hash.push_str(&format!("{:x}", x)),
            }
            hash.push_str("-");
        }

        if trace { println!("{}", hash); }

        return hash;
    }

    pub fn print_stats(&self, added: i32, total: i32) {
        println!("Trie is in {:?} mode. It has {} nodes. It contains {} unique paths out of {} total. Each node takes up 4x{} bytes so naive encoding totals {} bytes", BASE, self.nodes.len(), added, total, BASE as usize + 2, BASE as usize + 2 * 4 * self.nodes.len());
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
