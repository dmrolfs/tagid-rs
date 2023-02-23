use tailcall::tailcall;

pub fn encode(rep: &str) -> String {
    let mut base = rep.to_string();
    base.push_str(checksum(rep).to_string().as_str());
    base
}

#[allow(dead_code)]
pub fn decode(rep: &str) -> Option<&str> {
    if is_valid(rep) {
        rep.get(..(rep.len() - 1))
    } else {
        None
    }
}

pub fn is_valid(rep: &str) -> bool {
    checksum(rep) == 0
}

const MATRIX: [[usize; 10]; 10] = [
    [0, 3, 1, 7, 5, 9, 8, 6, 4, 2],
    [7, 0, 9, 2, 1, 5, 4, 8, 6, 3],
    [4, 2, 0, 6, 8, 7, 1, 3, 5, 9],
    [1, 7, 5, 0, 9, 8, 3, 4, 2, 6],
    [6, 1, 2, 3, 0, 4, 5, 9, 7, 8],
    [3, 6, 7, 4, 2, 0, 9, 5, 8, 1],
    [5, 8, 6, 9, 7, 2, 0, 1, 3, 4],
    [8, 9, 4, 5, 3, 6, 2, 0, 1, 7],
    [9, 4, 3, 8, 6, 1, 7, 2, 0, 5],
    [2, 5, 8, 1, 4, 3, 6, 7, 9, 0],
];

/// Calculates the checksum from the provided string
/// Params:
/// str – a string, only the numerics will be calculated
fn checksum(rep: &str) -> usize {
    do_checksum(rep.as_bytes(), 0, 0)
}

#[tailcall]
fn do_checksum(rep: &[u8], interim: usize, idx: usize) -> usize {
    if rep.len() <= idx {
        interim
    } else {
        let c = rep[idx] as char;
        let new_interim = if c.is_ascii_digit() {
            MATRIX[interim][c as usize - 48]
        } else {
            interim
        };
        do_checksum(rep, new_interim, idx + 1)
    }
}
