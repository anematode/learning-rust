// use std::arch::x86_64;

use std::cmp::max;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt;

// We store a position aligned to a 16-byte boundary for aligned loads. We order the bytes
// sequentially in memory, so that the first byte of Position (p.0 & 0xF) is the exponent of the
// top left tile. An exponent of 0, of course, means an empty tile. We distinguish between exponents
// and tiles; the 0 tile has an exponent of 0, the 2 tile has an exponent of 1, et cetera. Exponents
// are stored as (packed) u8s, while tiles are stored as u32s.
#[repr(C, align(16))]
struct Position(u64, u64);

fn exponent_to_tile(e: u8) -> u32 {
    if e == 0 {
        0
    } else {
        1u32 << e
    }
}

// Max theoretical is 2^17 = 131072
fn is_valid_exponent(e: u8) -> bool {
    e < 18
}

// Checks if exponent is in range and if it's a power of two
fn is_valid_tile(tile: u32) -> bool {
    let e = tile_to_exponent(tile);

    is_valid_exponent(e) && exponent_to_tile(e) == tile
}

// Convert a tile to an exponent; technically converts via a floored log, so 8,9 -> 4
fn tile_to_exponent(tile: u32) -> u8 {
    if tile == 0 {
        0
    } else {
        (31 - tile.leading_zeros()) as u8
    }
}

pub struct RNG {
    state: u64
}

impl RNG {
    pub fn next_u64(&mut self) {
        const M: u64 = 6364136223846793005;
        const I: u64 = 1;

        self.state = u64::wrapping_add(u64::wrapping_mul(M, self.state), I);
    }

    pub fn from_seed(seed: u64) -> RNG {
        RNG { state: seed }
    }
}

impl Position {
    // Get the exponent at a particular row and column
    pub fn exponent_at(&self, mut row: usize, col: usize) -> u8 {
        // Which u64 to read from
        let read = if row < 2 {
            self.0
        } else {
            row -= 2;
            self.1
        };

        // The shift to apply
        let shift = 8 * (row * 4 + col);

        ((read & (0xFu64 << shift)) >> shift) as u8
    }

    pub fn tile_at(&self, row: usize, col: usize) -> u32 {
        exponent_to_tile(self.exponent_at(row, col))
    }

    pub fn set_exponent(&mut self, mut row: usize, col: usize, e: u8) {
        let mut write = if row < 2 {
            &mut self.0
        } else {
            row -= 2;
            &mut self.1
        };

        let shift = 8 * (row * 4 + col);

        *write |= (0xFu64 & (e as u64)) << shift
    }

    pub fn set_tile(&mut self, row: usize, col: usize, tile: u32) {
        self.set_exponent(row, col, tile_to_exponent(tile));
    }

    // Panics if the position is invalid
    pub fn validate_position(&self) {
        for row in 0..4usize {
            for col in 0..4usize {
                let e = self.exponent_at(row, col);

                if !is_valid_exponent(e) {
                    panic!("Invalid exponent {e} at row {row} and column {col}", e=e, row=row, col=col);
                }
            }
        }
    }

    pub fn from_list(list: [u32; 16]) -> Position {
        let mut p = Position(0, 0);

        for row in 0..4 {
            for col in 0..4 {
                let i = row * 4 + col;
                let tile = list[i];

                if !is_valid_tile(tile) {
                    panic!("Invalid tile {tile} at index {i}", tile=tile, i=i);
                }

                p.set_tile(row, col, tile);
            }
        }

        p
    }

    pub fn to_string(&self) -> String {
        // Build a 4x4 array of strings
        let mut strs: [[String; 4]; 4] = Default::default();
        // Maximum width of each column
        let mut widths = [1usize; 4];

        for row in 0..4usize {
            for col in 0..4usize {
                strs[row][col] = self.tile_at(row, col).to_string();

                widths[col] = max(widths[col], strs[row][col].len());
            }
        }

        // Join the strings
        let mut out = String::new();

        for row in 0..4usize {
            for col in 0..4usize {
                let tile = &strs[row][col];

                // pad left with spaces
                out.push_str(&" ".repeat(widths[col] - tile.len()));
                out.push_str(tile);

                if col < 3 {
                    // column spacing
                    out += " ";
                }
            }

            out += "\n";
        }

        out
    }

    pub fn from_string(s: &str) -> Position {
        // Split s across whitespace
        let mut split = s.split_whitespace();
        let mut exponents = Vec::new();

        for (i, s) in split.enumerate() {
            let tile: u32 = s.parse().unwrap_or_else(|_| {
                panic!("Invalid tile {tile}", tile = s)
            });

            if !is_valid_tile(tile) {
                panic!("Invalid tile {tile} at index {i}", tile = tile, i = i)
            }

            exponents.push(tile_to_exponent(tile));
        }

        if exponents.len() != 16 {
            panic!("{count} tiles found (should be 16)", count = exponents.len())
        }

        let mut p = Position(0, 0);

        for row in 0..4 {
            for col in 0..4 {
                p.set_exponent(row, col, exponents[row * 4 + col]);
            }
        }

        p
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn main () {
    let test = Position::from_list([16u32, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    println!("{}", test);
}