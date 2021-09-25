use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;

use super::{ffi, Bitmap};

pub struct BatchedBitmapIterator<'a> {
    iterator: *mut ffi::roaring_uint32_iterator_s,
    buffer: [u32; 32],
    phantom: PhantomData<&'a ()>,
    done: bool,
}

impl<'a: 'b, 'b> BatchedBitmapIterator<'a> {
    fn new(bitmap: &Bitmap) -> Self {
        BatchedBitmapIterator {
            iterator: unsafe { ffi::roaring_create_iterator(bitmap.bitmap) },
            buffer: [0; 32],
            phantom: PhantomData,
            done: false,
        }
    }
    pub fn next(&'b mut self) -> Option<(u32, &'b [u32; 32])> {
        if self.done {
            return None;
        }
        let n = unsafe {
            ffi::roaring_read_uint32_iterator(self.iterator, self.buffer.as_mut_ptr(), 32)
        };
        if n == 0 {
            self.done = true;
            None
        } else {
            if n < 32 {
                self.done = true;
            }
            Some((n, &self.buffer))
        }
    }
}

pub struct BitmapIterator<'a> {
    iterator: *mut ffi::roaring_uint32_iterator_s,
    phantom: PhantomData<&'a ()>,
}

impl<'a> BitmapIterator<'a> {
    fn new(bitmap: &Bitmap) -> Self {
        BitmapIterator {
            iterator: unsafe { ffi::roaring_create_iterator(bitmap.bitmap) },
            phantom: PhantomData,
        }
    }

    #[inline]
    fn current_value(&self) -> Option<u32> {
        unsafe {
            if self.has_value() {
                Some((*self.iterator).current_value)
            } else {
                None
            }
        }
    }

    #[inline]
    fn has_value(&self) -> bool {
        unsafe { (*self.iterator).has_value }
    }

    #[inline]
    fn advance(&mut self) -> bool {
        unsafe { ffi::roaring_advance_uint32_iterator(self.iterator) }
    }
}

impl<'a> Iterator for BitmapIterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_value() {
            Some(value) => {
                self.advance();

                Some(value)
            }
            None => None,
        }
    }
}

impl<'a> Drop for BitmapIterator<'a> {
    fn drop(&mut self) {
        unsafe { ffi::roaring_free_uint32_iterator(self.iterator) }
    }
}

impl Bitmap {
    /// Returns an iterator over each value stored in the bitmap.
    /// Returned values are ordered in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let mut bitmap = Bitmap::create();
    /// bitmap.add(4);
    /// bitmap.add(3);
    /// bitmap.add(2);
    /// let mut iterator = bitmap.iter();
    ///
    /// assert_eq!(iterator.next(), Some(2));
    /// assert_eq!(iterator.next(), Some(3));
    /// assert_eq!(iterator.next(), Some(4));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&self) -> BitmapIterator {
        BitmapIterator::new(self)
    }

    pub fn batched_iter(&self) -> BatchedBitmapIterator {
        BatchedBitmapIterator::new(self)
    }
}

impl FromIterator<u32> for Bitmap {
    /// Convenience method for creating bitmap from iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use croaring::Bitmap;
    ///
    /// let bitmap: Bitmap = (1..3).collect();
    ///
    /// assert!(!bitmap.is_empty());
    /// assert!(bitmap.contains(1));
    /// assert!(bitmap.contains(2));
    /// assert_eq!(bitmap.cardinality(), 2);
    /// ```
    fn from_iter<I: IntoIterator<Item = u32>>(iter: I) -> Self {
        Bitmap::of(&Vec::from_iter(iter))
    }
}
