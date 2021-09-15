mod bytes;
mod inline;
mod shared;

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops;

use shared::SharedString;

const TAG_SIZE: usize = 1;
const CAPACITY: usize = std::mem::size_of::<SharedString>() - TAG_SIZE;
type InlineString = inline::InlineString<CAPACITY>;
type StdString = std::string::String;

pub use bytes::Bytes;

#[derive(Clone)]
pub struct String(StringInner);

#[derive(Clone)]
enum StringInner {
    Empty,
    Inline(InlineString),
    Shared(SharedString),
}

impl String {
    /// Creates a new empty `String`.
    ///
    /// Given that the `String` is empty, this will not allocate any initial
    /// buffer. While that means that this initial operation is very
    /// inexpensive, it may cause excessive allocation later when you add
    /// data.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s = eztd_core::String::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self(StringInner::Empty)
    }

    /// Returns the length of this `String`, in bytes, not [`char`]s or
    /// graphemes. In other words, it may not be what a human considers the
    /// length of the string.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let a = eztd_core::String::from("foo");
    /// assert_eq!(a.byte_len(), 3);
    /// ```
    #[inline]
    pub fn byte_len(&self) -> usize {
        self.as_str().len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s = eztd_core::String::from("");
    /// assert!(s.is_empty());
    ///
    /// let s = eztd_core::String::from("not empty");
    /// assert!(!s.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    /// Returns the length of this `String`, in bytes, not [`char`]s or
    /// graphemes. In other words, it may not be what a human considers the
    /// length of the string.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let a = eztd_core::String::from("foo");
    /// assert_eq!(a.char_len(), 3);
    /// ```
    #[inline]
    pub fn char_len(&self) -> usize {
        self.as_str().chars().count()
    }

    #[inline]
    #[deprecated = "Use either `byte_len` or `char_len` to be more explicit on meaning"]
    pub fn len(&self) -> usize {
        self.byte_len()
    }

    /// Returns a subslice of `String`.
    ///
    /// This is the non-panicking alternative to indexing the `String`. Returns
    /// [`None`] whenever equivalent indexing operation would panic.
    ///
    /// # Examples
    ///
    /// ```
    /// let v = eztd_core::String::from("Hello World");
    ///
    /// assert_eq!(Some(eztd_core::String::from("Hell")), v.get(0..4));
    /// ```
    #[inline]
    pub fn get(&self, range: impl std::ops::RangeBounds<usize>) -> Option<Self> {
        match self.coerce_range(range) {
            Some(range) => self.as_str().get(range).map(|s| self.own_str(s)),
            None => Some(String::new()),
        }
    }

    /// Divide one string slice into two at an index.
    ///
    /// The argument, `mid`, should be a byte offset from the start of the
    /// string. It must also be on the boundary of a UTF-8 code point.
    ///
    /// The two slices returned go from the start of the string slice to `mid`,
    /// and from `mid` to the end of the string slice.
    ///
    /// To get mutable string slices instead, see the [`split_at_mut`]
    /// method.
    ///
    /// [`split_at_mut`]: str::split_at_mut
    ///
    /// # Panics
    ///
    /// Panics if `mid` is not on a UTF-8 code point boundary, or if it is
    /// past the end of the last code point of the string slice.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s = eztd_core::String::from("Per Martin");
    ///
    /// let (first, last) = s.split_at(3);
    ///
    /// assert_eq!("Per", first);
    /// assert_eq!(" Martin", last);
    /// ```
    #[inline]
    pub fn split_at(&self, mid: usize) -> (Self, Self) {
        let (left, right) = self.as_str().split_at(mid);
        (self.own_str(left), self.own_str(right))
    }

    /// An iterator over the bytes of a string slice.
    ///
    /// As a string slice consists of a sequence of bytes, we can iterate
    /// through a string slice by byte. This method returns such an iterator.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let mut bytes = eztd_core::String::from("bors").bytes();
    ///
    /// assert_eq!(Some(b'b'), bytes.next());
    /// assert_eq!(Some(b'o'), bytes.next());
    /// assert_eq!(Some(b'r'), bytes.next());
    /// assert_eq!(Some(b's'), bytes.next());
    ///
    /// assert_eq!(None, bytes.next());
    /// ```
    #[inline]
    pub fn bytes(&self) -> Bytes {
        Bytes::new(self.clone())
    }

    /// Returns a string slice with leading whitespace removed.
    ///
    /// 'Whitespace' is defined according to the terms of the Unicode Derived
    /// Core Property `White_Space`.
    ///
    /// # Text directionality
    ///
    /// A string is a sequence of bytes. `start` in this context means the first
    /// position of that byte string; for a left-to-right language like English or
    /// Russian, this will be left side, and for right-to-left languages like
    /// Arabic or Hebrew, this will be the right side.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s = " Hello\tworld\t";
    /// assert_eq!("Hello\tworld\t", s.trim_start());
    /// ```
    #[inline]
    #[must_use = "this returns the trimmed string as a new string, \
                  without modifying the original"]
    pub fn trim_start(&self) -> Self {
        self.own_str(self.as_str().trim_start())
    }

    /// Appends a given string onto the end of this `String`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s = eztd_core::String::from("foo");
    ///
    /// let s = s.join_str("bar");
    /// assert_eq!("foobar", s);
    ///
    /// let baz = eztd_core::String::from("baz");
    /// let s = s.join_str(baz);
    ///
    /// assert_eq!("foobarbaz", s);
    /// ```
    #[inline]
    #[must_use = "this returns the trimmed string as a new string, \
                  without modifying the original"]
    pub fn join_str(&self, string: impl AsRef<str>) -> Self {
        let mut buffer = StdString::from(self.as_str());
        buffer.push_str(string.as_ref());
        Self::from(buffer.as_str())
    }

    /// Appends the given [`char`] to the end of this `String`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s = eztd_core::String::from("abc");
    ///
    /// let s = s.join_char('1').join_char('2').join_char('3');
    ///
    /// assert_eq!("abc123", s);
    /// ```
    #[inline]
    #[must_use = "this returns the trimmed string as a new string, \
                  without modifying the original"]
    pub fn join_char(&self, ch: char) -> Self {
        let mut buffer = StdString::from(self.as_str());
        buffer.push(ch);
        Self::from(buffer.as_str())
    }

    /// Shrinks the capacity of this `String` to match its length.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s = eztd_core::String::from("foo");
    ///
    /// let s = s.shrink_to_fit();
    /// ```
    #[inline]
    #[must_use = "this returns the trimmed string as a new string, \
                  without modifying the original"]
    pub fn shrink_to_fit(&self) -> String {
        String::from(self.as_str())
    }

    fn own_str(&self, subset: &str) -> Self {
        if subset.is_empty() {
            String::new()
        } else {
            match &self.0 {
                StringInner::Empty => String::new(),
                StringInner::Inline(s) => s.own_str(subset).into(),
                StringInner::Shared(s) => s.own_str(subset).into(),
            }
        }
    }

    fn coerce_range(
        &self,
        range: impl std::ops::RangeBounds<usize>,
    ) -> Option<std::ops::RangeInclusive<usize>> {
        let len = self.byte_len();
        if len == 0 {
            return None;
        }

        let range_start = match range.start_bound() {
            std::ops::Bound::Included(s) => *s,
            std::ops::Bound::Excluded(s) => {
                if *s == usize::MAX {
                    return None;
                } else {
                    s + 1
                }
            }
            std::ops::Bound::Unbounded => 0,
        };
        let range_end = match range.end_bound() {
            std::ops::Bound::Included(s) => *s,
            std::ops::Bound::Excluded(s) => {
                if *s == 0 {
                    return None;
                } else {
                    s - 1
                }
            }
            std::ops::Bound::Unbounded => usize::MAX,
        }
        .min(len - 1);

        if len <= range_start || range_end < range_start {
            None
        } else {
            Some(range_start..=range_end)
        }
    }
}

/// Transitional Python API
impl String {
    #[deprecated = "In Rust, we refer to this as `trim_start`"]
    pub fn lstrip(&self) -> Self {
        self.trim_start()
    }
}

/// Interop
impl String {
    /// Extracts a string slice containing the entire `String`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let s = eztd_core::String::from("foo");
    ///
    /// assert_eq!("foo", s.as_str());
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            StringInner::Empty => "",
            StringInner::Inline(s) => s.as_str(),
            StringInner::Shared(s) => s.as_str(),
        }
    }
}

pub(crate) fn calculate_subset(s: &str, subset: &str) -> (usize, usize) {
    unsafe {
        let self_start = s.as_ptr();
        let self_end = self_start.add(s.len());

        let subset_start = subset.as_ptr();
        let subset_end = subset_start.add(subset.len());
        debug_assert!(self_start <= subset_start);
        debug_assert!(subset_end <= self_end);

        let start = subset_start.offset_from(self_start) as usize;
        let end = subset_end.offset_from(self_start) as usize;
        (start, end)
    }
}

impl Default for String {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<InlineString> for String {
    #[inline]
    fn from(other: InlineString) -> Self {
        Self(StringInner::Inline(other))
    }
}

impl From<SharedString> for String {
    #[inline]
    fn from(other: SharedString) -> Self {
        Self(StringInner::Shared(other))
    }
}

impl<'s> From<&'s str> for String {
    #[inline]
    fn from(other: &'s str) -> Self {
        match other.len() {
            0 => String::new(),
            len if len <= CAPACITY => InlineString::from(other).into(),
            _ => SharedString::from(other).into(),
        }
    }
}

impl From<StdString> for String {
    #[inline]
    fn from(other: StdString) -> Self {
        other.as_str().into()
    }
}

impl From<char> for String {
    #[inline]
    fn from(other: char) -> Self {
        Self::new().join_char(other)
    }
}

impl std::str::FromStr for String {
    type Err = core::convert::Infallible;
    #[inline]
    fn from_str(s: &str) -> Result<String, Self::Err> {
        Ok(String::from(s))
    }
}

impl<'s> From<&'s StdString> for String {
    #[inline]
    fn from(other: &'s StdString) -> Self {
        other.as_str().into()
    }
}

impl FromIterator<char> for String {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> String {
        let s = StdString::from_iter(iter);
        String::from(&s)
    }
}

impl<'a> FromIterator<&'a char> for String {
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> String {
        let s = StdString::from_iter(iter);
        String::from(&s)
    }
}

impl<'a> FromIterator<&'a str> for String {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> String {
        let s = StdString::from_iter(iter);
        String::from(&s)
    }
}

impl FromIterator<StdString> for String {
    fn from_iter<I: IntoIterator<Item = StdString>>(iter: I) -> String {
        let s = StdString::from_iter(iter);
        String::from(&s)
    }
}

/// Implements the `+` operator for concatenating two strings.
///
/// This consumes the `String` on the left-hand side and re-uses its buffer (growing it if
/// necessary). This is done to avoid allocating a new `String` and copying the entire contents on
/// every operation, which would lead to *O*(*n*^2) running time when building an *n*-byte string by
/// repeated concatenation.
///
/// The string on the right-hand side is only borrowed; its contents are copied into the returned
/// `String`.
///
/// # Examples
///
/// Concatenating two `String`s takes the first by value and borrows the second:
///
/// ```
/// let a = eztd_core::String::from("hello");
/// let b = eztd_core::String::from(" world");
/// let c = &a + &b + "foo";
/// ```
impl<'s, S: AsRef<str>> std::ops::Add<S> for &'s String {
    type Output = String;

    #[inline]
    fn add(self, other: S) -> String {
        let other = other.as_ref();
        self.join_str(other)
    }
}
impl<S: AsRef<str>> std::ops::Add<S> for String {
    type Output = String;

    #[inline]
    fn add(self, other: S) -> String {
        let other = other.as_ref();
        self.join_str(other)
    }
}

// TODO: Determine policy
// - Should we index by bytes or chars?
// - Should we do python-style negative numbers?
impl ops::Index<ops::Range<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::Range<usize>) -> &str {
        self.coerce_range(index)
            .map(|index| &self.as_str()[index])
            .unwrap_or_default()
    }
}
impl ops::Index<ops::RangeTo<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeTo<usize>) -> &str {
        self.coerce_range(index)
            .map(|index| &self.as_str()[index])
            .unwrap_or_default()
    }
}
impl ops::Index<ops::RangeFrom<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeFrom<usize>) -> &str {
        self.coerce_range(index)
            .map(|index| &self.as_str()[index])
            .unwrap_or_default()
    }
}
impl ops::Index<ops::RangeFull> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeFull) -> &str {
        self.coerce_range(index)
            .map(|index| &self.as_str()[index])
            .unwrap_or_default()
    }
}
impl ops::Index<ops::RangeInclusive<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeInclusive<usize>) -> &str {
        self.coerce_range(index)
            .map(|index| &self.as_str()[index])
            .unwrap_or_default()
    }
}
impl ops::Index<ops::RangeToInclusive<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: ops::RangeToInclusive<usize>) -> &str {
        self.coerce_range(index)
            .map(|index| &self.as_str()[index])
            .unwrap_or_default()
    }
}

impl fmt::Display for String {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl fmt::Debug for String {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for String {}

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }
    };
}

impl_eq! { String, str }
impl_eq! { String, &'a str }
impl_eq! { String, StdString }
impl_eq! { String, &'a StdString }

impl PartialOrd for String {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for String {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for String {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl AsRef<str> for String {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod test_coerce_range {
    use super::*;

    #[test]
    fn empty() {
        let fixture = "";
        let outside = 10;
        assert_eq!(String::from(fixture).coerce_range(..), None);
        assert_eq!(String::from(fixture).coerce_range(0..), None);
        assert_eq!(String::from(fixture).coerce_range(outside..), None);
        assert_eq!(String::from(fixture).coerce_range(..outside), None);
        assert_eq!(String::from(fixture).coerce_range(..0), None);
        assert_eq!(String::from(fixture).coerce_range(0..0), None);
        assert_eq!(String::from(fixture).coerce_range(0..outside), None);
        assert_eq!(String::from(fixture).coerce_range(outside..0), None);
        assert_eq!(String::from(fixture).coerce_range(0..=0), None);
        assert_eq!(String::from(fixture).coerce_range(0..=outside), None);
        assert_eq!(String::from(fixture).coerce_range(outside..=0), None);
    }

    #[test]
    fn non_empty() {
        let fixture = "Hello";
        let inside = 3;
        assert!(inside < fixture.len());
        let outside = 10;
        assert!(fixture.len() < outside);

        assert_eq!(String::from(fixture).coerce_range(..), Some(0..=4));
        assert_eq!(String::from(fixture).coerce_range(0..), Some(0..=4));
        assert_eq!(String::from(fixture).coerce_range(inside..), Some(3..=4));
        assert_eq!(String::from(fixture).coerce_range(outside..), None);
        assert_eq!(String::from(fixture).coerce_range(..inside), Some(0..=2));
        assert_eq!(String::from(fixture).coerce_range(..outside), Some(0..=4));
        assert_eq!(String::from(fixture).coerce_range(..0), None);
        assert_eq!(String::from(fixture).coerce_range(0..0), None);
        assert_eq!(String::from(fixture).coerce_range(0..inside), Some(0..=2));
        assert_eq!(String::from(fixture).coerce_range(0..outside), Some(0..=4));
        assert_eq!(
            String::from(fixture).coerce_range(inside..outside),
            Some(3..=4)
        );
        assert_eq!(String::from(fixture).coerce_range(inside..0), None);
        assert_eq!(String::from(fixture).coerce_range(outside..0), None);
        assert_eq!(String::from(fixture).coerce_range(outside..inside), None);
        assert_eq!(String::from(fixture).coerce_range(0..=0), Some(0..=0));
        assert_eq!(String::from(fixture).coerce_range(0..=inside), Some(0..=3));
        assert_eq!(String::from(fixture).coerce_range(0..=outside), Some(0..=4));
        assert_eq!(
            String::from(fixture).coerce_range(inside..=outside),
            Some(3..=4)
        );
        assert_eq!(String::from(fixture).coerce_range(inside..=0), None);
        assert_eq!(String::from(fixture).coerce_range(outside..=0), None);
        assert_eq!(String::from(fixture).coerce_range(outside..=inside), None);
    }
}
