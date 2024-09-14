---
title: "Using Rust Trait AsRef along with Generic Data Type"
date: 2024-09-14T12:40:49+08:00
---

<!-- vim-markdown-toc GFM -->

* [Quiz](#quiz)
* [Tips](#tips)
    * [`IntoIterator` and `Iterator` Trait](#intoiterator-and-iterator-trait)
    * [Generic Data Type and `AsRef` Trait](#generic-data-type-and-asref-trait)
* [Example answer](#example-answer)
* [Follow-up: Why `bit_iter` is read only while the `next()` need `&mut self`](#follow-up-why-bit_iter-is-read-only-while-the-next-need-mut-self)
* [Follow-up: Can I modify the data during iterator](#follow-up-can-i-modify-the-data-during-iterator)
* [Follow-up: Why not use `From` Trait?](#follow-up-why-not-use-from-trait)

<!-- vim-markdown-toc -->

The Rust Trait is one of my most favor features in Rust and also
the biggest obstacle I faced during my rust learning path.
Whenever I fell I have mastered it by reading Rust books and blogs,
the compiler still politely remind me the opposite with flooding errors.

So let's learn it in a complex way through a coding quiz.

### Quiz

Providing `main.rs` with this content:

```rust
// SPDX-License-Identifier: Apache-2.0

mod bit_iterator;

use self::bit_iterator::BitIterator;

fn main() {
    let a = vec![1u8, 2, 4, 8];
    let bit_iter = BitIterator::new(&a);

    for pos in bit_iter {
        println!("Got bit {pos} set to 1");
    }
}
```

Please implement `BitIterator` in `bit_iterator.rs` counting the position
of bit set to 1.

The expected output is

```
Got bit 0 set to 1
Got bit 9 set to 1
Got bit 18 set to 1
Got bit 27 set to 1
```

### Tips

#### `IntoIterator` and `Iterator` Trait

The for loop is actually a syntax sugar using `IntoIterator` trait.
And the [rust doc for iterator][iter_moduel_doc] mentioned the `IntoIterator`
is automatically implement if your data type has implemented `Iterator`:

```rust
impl<I: Iterator> IntoIterator for I
```

Now, our task is clear: implement [std::iter::Iterator][iter_doc] with:

 * `type Item` indicate the return data type on each iteration.
 * `fn next() -> Option<Self::Item>` to iterate. None will stop the iteration.

#### Generic Data Type and `AsRef` Trait

The `BitIterator::new()` is taking a reference, if we are storing it
into `struct BitIterator`, we need to define its lifetime which is another
nightmare for rust new learner.

Let avoid it by using `AsRef` and generic data type `T`(It does not have to be
named as `T`, just a convention) like:

```rust
pub(crate) struct BitIterator<T: AsRef<[u8]>> {
    data: T,
    index: usize,
    pos: u8,
}
```

With that, we can use `let data: &[u8] = self.data.as_ref()` to access the
data to search bits.

Now you can implement the Iterator Trait like:

```rust
impl<T: AsRef<[u8]>> std::iter::Iterator for BitIterator<T> {
}
```

Now, you got everything required. Please stop the reading and code the quiz
out. Try to understand compiler errors you got during coding and seek for help
from search engine, or other developers.


### Example answer

```rust
// SPDX-License-Identifier: Apache-2.0

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct BitIterator<T: AsRef<[u8]>> {
    data: T,
    index: usize,
    pos: u8,
}

impl<T: AsRef<[u8]>> BitIterator<T> {
    pub(crate) fn new(data: T) -> Self {
        Self {
            data,
            index: 0,
            pos: 0,
        }
    }

    fn is_cur_bit_set(&self) -> bool {
        self.data
            .as_ref()
            .get(self.index)
            .map(|byte| (byte & 1 << self.pos) >= 1)
            .unwrap_or(false)
    }

    fn next_bit(&mut self) -> Result<(), ()> {
        let data: &[u8] = self.data.as_ref();
        if self.index >= data.len()
            || (self.pos == 7 && self.index == data.len() - 1)
        {
            return Err(());
        }
        if self.pos == 7 {
            self.pos = 0;
            self.index += 1;
        } else {
            self.pos += 1;
        }
        Ok(())
    }
}

impl<T: AsRef<[u8]>> std::iter::Iterator for BitIterator<T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.is_cur_bit_set() {
                let cur_pos = self.index * 8 + self.pos as usize;
                self.next_bit().ok();
                return Some(cur_pos);
            }
            if self.next_bit().is_err() {
                return None;
            }
        }
    }
}
```

I believe the code is transparent, but I would like to bring up some questions
to dive deep into this topic.

### Follow-up: Why `bit_iter` is read only while the `next()` need `&mut self`

The `for` loop is syntax sugar, meaning this `for pos in bit_iter` is actually
looks like(not real compiler de-sugar code):

```rust
{
    let mut iter = IntoIterator::into_iter(bit_iter);
    loop {
        match iter.next() {
            Some(item) => {
                // you code in `for` loop
            }
            None => break;
        }
    }
}
```

The `IntoIterator::into_iter()` took the ownership and change it to `mut`.


### Follow-up: Can I modify the data during iterator

You may try to compile this:

```rust
// SPDX-License-Identifier: Apache-2.0

mod bit_iterator;

use self::bit_iterator::BitIterator;

fn main() {
    let mut a = vec![1u8, 2, 4, 8];
    let bit_iter = BitIterator::new(&a);

    for pos in bit_iter {
        a[1] = 4;
        println!("Got bit {pos} set to 1");
    }

    println!("HAHA {:?}", a);
}
```

The compiler will stop you with

```
cannot borrow `a` as mutable because it is also borrowed as immutable.
```

If you want to modify it, please store the pending changes and modify it
after the iterator drooped, for example:


```
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

    // The bit_iter is droped here because of `IntoIterator()` mentioned above.

    for change in pending_changes {
        a.push(change);
    }

    println!("HAHA {:?}", a);
}
```

### Follow-up: Why not use `From` Trait?

Instead of `data.as_ref()`, we can only use `let data: &[u8] = data.into()`.

It still works well. Both `AsRef` and `From` can convert one data type to
another. But the `AsRef` trait is designed for `cheap reference-to-reference`
while the `From` trait is targeted for costly conversion. In our case
`Vec<u8>` to `&[u8]` is a cheap reference-to-reference conversion.

Meanwhile, many rust native data type as `AsRef<[u8]>` implemented, but no
`Into<[u8]>`. Using `AsRef` allowing our iterator support more data types
out of box.

Hence, `AsRef()` is preferred here.

[iter_module_doc]: https://doc.rust-lang.org/std/iter/
[iter_doc]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
