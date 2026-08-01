//! A Ring Buffer implementation.
//! A ring buffer is a type of buffer with a set length that will
//! overwrite the oldest values it holds when new ones are added.
//!
//! Modified from my [radio-data](https://github.com/connorslade/radio-data/blob/master/src/misc/ring_buffer.rs) project.

use core::mem::MaybeUninit;

/// Ring buffer that can hold any type.
/// The size of the buffer is defined as SIZE at compile time so it can be stored on the stack.
pub struct RingBuffer<T, const SIZE: usize> {
    pub data: [MaybeUninit<T>; SIZE],
    pub index: usize,
    pub filled: bool,
}

impl<T: Default + Copy, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Create a new RingBuffer using T::default().
    pub fn new() -> Self {
        Self {
            data: [const { MaybeUninit::uninit() }; SIZE],
            index: 0,
            filled: false,
        }
    }
}

impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Adds a new value to the buffer
    pub fn push(&mut self, val: T) {
        self.data[self.index].write(val);
        let idx = self.index + 1;
        self.index = idx % SIZE;

        if !self.filled && idx == SIZE {
            self.filled = true;
        }
    }

    /// Gets the values that have actually been set.
    /// If self.filled is true, this will be the whole buffer,
    /// if not it will just be the values added by the user.
    fn real(&self) -> &[MaybeUninit<T>] {
        if self.filled {
            return &self.data;
        }

        &self.data[..self.index]
    }
}

impl<const SIZE: usize> RingBuffer<f32, SIZE> {
    /// Get the average of the values from the buffer.
    pub fn avg(&self) -> f32 {
        let real = self.real();
        let sum = real
            .iter()
            .fold(0.0, |a, &b| a + unsafe { b.assume_init() });
        sum / real.len() as f32
    }
}
