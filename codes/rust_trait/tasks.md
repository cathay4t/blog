## Create BitIterator for `Vec<u8>`

Please create `bit_iterator.rs` to allow below code print out

        Got bit 0 set to 1
        Got bit 5 set to 1
        Got bit 10 set to 1
        GOt bit 15 set to 1


```rust
mod bit_iterator;

use self::bit_iterator::BitIterator;

fn main() {
    let a = vec![1u8, 2, 4, 8];
    let mut bit_iter = BitIterator::new(a);

    for localtion in bit_tier {
        println!("Got bit {} set to 1", a);
    }
}
```

## BitIterator::new() accept `Vec<u8>`, `&[u8]`, `String`, `&str` and etc


Change `bit_iterator.rs` to accept `&[u8]`, `&str`, `String` in
`BitIterator::new()`.


## Restrict with another Trait
