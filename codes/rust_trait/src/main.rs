// SPDX-License-Identifier: Apache-2.0

mod bit_iterator;

use self::bit_iterator::BitIterator;

fn main() {
    let mut a = vec![1u8, 2, 4, 8];
    let bit_iter = BitIterator::new(&a);
    let mut pending_changes: Vec<u8> = Vec::new();

    for pos in bit_iter {
        pending_changes.push(pos as u8);
        println!("Got bit {pos} set to 1");
    }

    for change in pending_changes {
        a.push(change);
    }

    println!("HAHA {:?}", a);
}
