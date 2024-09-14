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
