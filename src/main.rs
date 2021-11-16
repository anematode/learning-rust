use std::cmp::max;

type Tile = u32;      // Actual value of a tile (0, 2, 4, ..., 131072)
type PackedTile = u8; // Packed value, represent 0 as 0b00000000, 2 as 0b10000001, and in general 2^x as 128 + x
type Position = [[Tile; 4]; 4];

// Packed from top left to bottom right but in little endian order; first u64 stores the last two rows
#[repr(C, align(16))]
struct PackedPosition(u64, u64);

fn empty_position() -> Position {
    [[0; 4]; 4]
}

fn empty_packed_position() -> PackedPosition {
    PackedPosition(0, 0)
}

fn pack_tile(tile: Tile) -> PackedTile {
    match tile {
        0 => 0,
        _ => 0x80 + (31 - tile.leading_zeros()) as u8
    }
}

fn unpack_tile(packed_tile: PackedTile) -> Tile {
    match packed_tile {
        0 => 0,
        _ => 1 << (packed_tile - 0x80)
    }
}

fn unpack_position(packed_position: PackedPosition) -> Position {
    let PackedPosition(first, second) = packed_position;

    let mut tiles: Position = empty_position();

    for byte in 0..8 {
        let shift = 8 * (7 - byte);
        let mask = 0xFFu64 << shift;

        tiles[2 + (byte >> 2)][byte & 0b11] = unpack_tile(((first & mask) >> shift) as PackedTile);
        tiles[byte >> 2][byte & 0b11] = unpack_tile(((second & mask) >> shift) as PackedTile);
    }

    tiles
}

// Pack a position
fn pack_position(position: &Position) -> PackedPosition {
    let mut first = 0u64;
    let mut second = 0u64;

    for i in 0..4usize {
        for j in 0..4usize {
            let shift = 8 * (7 - 4 * (i % 2) - j);
            let pack = (pack_tile(position[i][j]) as u64) << shift;

            if i < 2 {
                second += pack;
            } else {
                first += pack;
            }
        }
    }

    PackedPosition(first, second)
}

// Convert position to readable string
fn position_to_string(position: &Position) -> String {
    let mut out_arr: [[String; 4]; 4] = Default::default();
    let mut max_width = 1;

    for row in 0..4usize {
        for col in 0..4usize {
            let str = position[row][col].to_string();
            let len = str.chars().count();

            max_width = max(max_width, len);

            out_arr[row][col] = str;
        }
    }

    let mut out = String::new();

    for row in 0..4usize {
        for col in 0..4usize {
            let str = &out_arr[row][col];

            out.push_str(str);
            out.push_str(&" ".repeat(max_width - str.chars().count() + 1))
        }
        out.push_str("\n");
    }

    out
}

fn print_position(position: &Position) {
    println!("{}", position_to_string(position));
}

use std::arch::x86_64::*;

unsafe fn _load_packed_position(packed_position: PackedPosition) -> __m128i {
    return _mm_load_si128(std::ptr::addr_of!(packed_position) as *const __m128i);
}

// Position           Byte indices
// 1  2  3  4         15 14 13 12
// 5  6  7  8         11 10 9 8
// 9  10 11 12        7  6  5  4
// 13 14 15 16        3  2  1  0

unsafe fn _rotate_packed_position(pos: __m128i, mut quarter_turns: i32) -> __m128i {
    let mask = match quarter_turns {
        1 => {
            _mm_set_epi8(3, 7, 11, 15, 2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12)
        },
        2 => {
            _mm_set_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15)
        },
        3 => {
            _mm_set_epi8(12, 8, 4, 0, 13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3)
        },
        0 | _ => {
            return pos;
        }
    };

    _mm_shuffle_epi8(pos, mask)
}

fn _get_packed_position(pos: __m128i) -> PackedPosition {
    let mut result: PackedPosition = PackedPosition(0, 0);

    unsafe {
        // Write to result
        _mm_store_si128(std::ptr::addr_of!(result) as *mut __m128i, pos);
    }

    result
}


// 0000 represents

fn main() {
    let mut position: Position = [
        [16, 8, 8, 4],
        [4, 2, 0, 0],
        [2, 0, 0, 0],
        [0, 0, 2, 0]
    ];

    let packed = pack_position(&position);
    
    unsafe {
        let new_packed = _get_packed_position(_load_packed_position(packed));

        print_position(&unpack_position(new_packed));
    }
}
