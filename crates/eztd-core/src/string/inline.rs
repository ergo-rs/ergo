use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone)]
pub(crate) struct InlineString<const N: usize> {
    len: u8,
    array: [u8; N],
}

impl<const N: usize> InlineString<N> {
    pub(crate) fn own_str(&self, subset: &str) -> Self {
        let (start, end) = super::calculate_subset(self.as_str(), subset);
        Self::from(&self.as_str()[start..end])
    }

    pub(crate) fn as_str(&self) -> &str {
        let len = self.len as usize;
        unsafe { std::str::from_utf8_unchecked(&self.array[..len]) }
    }
}

impl<const N: usize> fmt::Debug for InlineString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> PartialEq for InlineString<N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> Eq for InlineString<N> {}

impl<const N: usize> PartialOrd for InlineString<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for InlineString<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<const N: usize> Hash for InlineString<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'s, const N: usize> From<&'s str> for InlineString<N> {
    fn from(other: &'s str) -> Self {
        let b = other.as_bytes();
        let len = b.len();
        debug_assert!(len <= N);

        let mut array = [0; N];
        array[..len].copy_from_slice(b);
        Self {
            len: len as u8,
            array,
        }
    }
}
