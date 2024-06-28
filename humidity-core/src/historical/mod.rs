use crate::serde;

pub struct Historical<const SIZE: usize, T> {
    elements: [Option<T>; SIZE],
    len: usize,
}

impl<const SIZE: usize, T> Default for Historical<SIZE, T>
where
    T: serde::Serializable<T>,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize, T> Historical<SIZE, T>
where
    T: serde::Serializable<T>,
{
    const EMPTY: Option<T> = Option::None;

    pub const fn new() -> Self {
        Self { elements: [Self::EMPTY; SIZE], len: 0 }
    }

    pub fn store(&mut self, elem: T) {
        self.elements[self.next()] = Some(elem);
    }

    pub fn sync(&self) -> Syncer<T> {
        Syncer::new(&self.elements[..self.len])
    }

    fn next(&mut self) -> usize {
        let next = self.len;
        self.len += 1;
        if self.len == SIZE {
            self.len = 0;
        }
        next
    }
}

pub struct Syncer<'out, T> {
    elements: &'out [Option<T>],
    pos: usize,
}

impl<'out, T> Syncer<'out, T>
where
    T: serde::Serializable<T>,
{
    fn new(elements: &'out [Option<T>]) -> Self {
        Self { elements, pos: 0 }
    }

    pub fn write(&mut self, out: &mut [u8]) -> Result<usize, serde::Error> {
        if self.pos >= self.elements.len() {
            return Ok(0);
        }
        let elem = &self.elements[self.pos];
        if let Some(elem) = elem {
            self.pos += 1;
            return serde::serialize(elem, out);
        }
        Ok(0)
    }
}
