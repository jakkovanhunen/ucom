use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum Vec1Err {
    VecIsEmpty,
    CannotPopLastElement,
}

/// A non-empty list.
pub struct Vec1<T> {
    inner: Vec<T>,
}

impl<T> DerefMut for Vec1<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Deref for Vec1<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[allow(dead_code)]
impl<T> Vec1<T> {
    /// Creates a new non-empty list.
    pub fn new(first: T) -> Self {
        let inner = vec![first];
        Self { inner }
    }

    /// Creates a new non-empty list from a vector.
    pub fn from_vec(value: Vec<T>) -> Result<Self, Vec1Err> {
        if value.is_empty() {
            Err(Vec1Err::VecIsEmpty)
        } else {
            Ok(Self { inner: value })
        }
    }

    /// Converts the list into a vector.
    pub fn into_vec(self) -> Vec<T> {
        self.inner
    }

    /// Returns the length of the list.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the first value.
    pub fn first(&self) -> &T {
        &self.inner[0]
    }

    /// Returns the last value.
    pub fn last(&self) -> &T {
        self.inner
            .last()
            .expect("NonEmptyVec should never be empty")
    }

    /// Returns a mutable reference to the first value.
    pub fn first_mut(&mut self) -> &mut T {
        &mut self.inner[0]
    }

    /// Returns a mutable reference to the last value.
    pub fn last_mut(&mut self) -> &mut T {
        self.inner
            .last_mut()
            .expect("NonEmptyVec should never be empty")
    }

    /// Returns a mutable iterator over the values.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.inner.iter_mut()
    }

    /// Pushes a value to the end of the list.
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// Pops the last value from the list.
    /// Returns an error if attempting to pop the last value.
    pub fn pop(&mut self) -> Result<T, Vec1Err> {
        if self.inner.len() == 1 {
            Err(Vec1Err::CannotPopLastElement)
        } else {
            Ok(self.inner.pop().unwrap())
        }
    }

    /// Appends the values from the other list.
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.inner.append(other);
    }

    /// Extends the list with the values from the other list.
    pub fn extend(&mut self, other: Vec<T>) {
        self.inner.extend(other);
    }
}

impl<T> TryFrom<Vec<T>> for Vec1<T> {
    type Error = Vec1Err;

    /// Tries to create a non-empty list from a vector.
    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(Vec1Err::VecIsEmpty)
        } else {
            Ok(Self { inner: value })
        }
    }
}

#[allow(clippy::from_over_into)]
impl<T> Into<Vec<T>> for Vec1<T> {
    fn into(self) -> Vec<T> {
        self.inner
    }
}

impl<T> AsRef<Vec<T>> for Vec1<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.inner
    }
}
