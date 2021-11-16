use std::arch::x86_64;
use std::cmp::max;

type Tile = u32;      // Actual value of a tile (0, 2, 4, ..., 131072)
type PackedTile = u8; // Packed value, represent 0 as 0b00000000, 2 as 0b10000001, and in general 2^x as 128 + x
type Position = [[Tile; 4]; 4];

// Packed from top left to bottom right, rows first
type PackedPosition = [u64; 2]; // probably better to use this instead of a u128

fn empty_position() -> Position {
    [[0; 4]; 4]
}

fn empty_packed_position() -> PackedPosition {
    [0, 0]
}

fn pack_tile(tile: Tile) -> PackedTile {
    match tile {
        0 => 0,
        _ => 0x80 + (31 - tile.leading_zeros()) as u8
    }
}

fn unpack_tile(packed_tile: PackedTile) -> Tile {
    println!("{}", packed_tile);

    match packed_tile {
        0 => 0,
        _ => 1 << (packed_tile - 0x80)
    }
}

fn unpack_position(packed_position: PackedPosition) -> Position {
    let [ first, second ] = packed_position; // destructuring :)

    let mut tiles: Position = empty_position();

    for byte in 0..8 {
        let shift = 8 * (7 - byte);
        let mask = 0xFFu64 << shift;

        println!("{} {}", first, mask);

        tiles[byte >> 2][byte & 0b11] = unpack_tile(((first & mask) >> shift) as PackedTile);
        tiles[2 + (byte >> 2)][byte & 0b11] = unpack_tile(((second & mask) >> shift) as PackedTile);
    }

    return tiles;
}

// Pack a position into two 64 bit integers
fn pack_position(position: &Position) -> PackedPosition {
    let mut first = 0u64;
    let mut second = 0u64;

    for i in 0..4usize {
        for j in 0..4usize {
            let shift = 8 * (7 - 4 * (i % 2) - j);
            let pack = (pack_tile(position[i][j]) as u64) << shift;

            if i < 2 {
                first += pack;
            } else {
                second += pack;
            }
        }
    }

    return [first, second];
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

    return out;
}

fn print_position(position: &Position) {
    println!("{}", position_to_string(position));
}

// 0000 represents

fn main() {
    let mut position: Position = [
        [16, 8, 8, 4],
        [4, 0, 0, 0],
        [2, 0, 2, 0],
        [0, 0, 0, 0]
    ];

    let packed = pack_position(&position);
    println!("{}", packed[0]);
    let unpacked = unpack_position(packed);


    print_position(&position);
    print_position(&unpacked);
}
