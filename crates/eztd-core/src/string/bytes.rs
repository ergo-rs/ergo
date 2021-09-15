pub struct Bytes {
    buffer: super::String,
    index: usize,
}

impl Bytes {
    pub(super) fn new(buffer: super::String) -> Self {
        Self { buffer, index: 0 }
    }
}

impl Iterator for Bytes {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.byte_len() {
            let current = self.index;
            self.index += 1;
            Some(self.buffer.as_str().as_bytes()[current])
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.buffer.byte_len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for Bytes {
    #[inline]
    fn len(&self) -> usize {
        self.buffer.byte_len()
    }
}

impl std::iter::FusedIterator for Bytes {}
