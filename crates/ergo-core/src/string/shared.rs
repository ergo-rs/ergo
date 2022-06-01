use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct SharedString {
    buffer: Arc<str>,
    start: usize,
    len: usize,
}

impl SharedString {
    pub(crate) fn own_str(&self, subset: &str) -> Self {
        let (start, end) = super::calculate_subset(self.as_str(), subset);
        let len = end - start;
        let mut sub = self.clone();
        sub.start = start;
        sub.len = len;
        sub
    }

    pub(crate) fn as_str(&self) -> &str {
        let end = self.start + self.len;
        &self.buffer.as_ref()[self.start..end]
    }
}

impl fmt::Debug for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl PartialEq for SharedString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for SharedString {}

impl PartialOrd for SharedString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SharedString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for SharedString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'s> From<&'s str> for SharedString {
    fn from(other: &'s str) -> Self {
        Self {
            buffer: Arc::from(other),
            start: 0,
            len: other.len(),
        }
    }
}
