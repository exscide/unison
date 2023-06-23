//! Some useful types.

/// A growable ring buffer.
/// 
/// Values can be pushed into the buffer via [RingBuffer::push]
/// and popped via [RingBuffer::pop_top] and [RingBuffer::pop_bottom].
/// 
/// When the buffer is full, its capacity will grow.
/// Growing will shift values around so having a larger capacity is preferred.
pub struct RingBuffer<T> {
	buffer: Vec<std::mem::MaybeUninit<T>>,
	head: usize,
	tail: usize,
}

impl<T> RingBuffer<T> {
	pub fn new(capacity: usize) -> Self {
		Self {
			buffer: (0..capacity).into_iter().map(|_| std::mem::MaybeUninit::uninit()).collect(),
			head: 0,
			tail: 0,
		}
	}

	/// Push a value into the buffer.
	/// If the buffer is full, increase its capacity.
	pub fn push(&mut self, val: T) {
		let val = std::mem::MaybeUninit::new(val);

		let mut next = self.head + 1;

		// wrap around
		if next >= self.buffer.len() { next = 0; }

		if next == self.tail {
			// buffer is full, grow
			self.buffer.insert(self.head, val);

			// all elements to the right will be shifted by one, increase the tail
			self.tail += 1;
		} else {
			self.buffer[self.head] = val;
		}

		self.head = next
	}

	/// Pop a value from the head of the buffer.
	/// Returns [None] if the buffer is empty.
	pub fn pop_top(&mut self) -> Option<T> {
		if self.is_empty() {
			return None;
		}

		if self.head == 0 {
			// wrap around
			self.head = self.buffer.len() - 1;
		} else {
			self.head -= 1;
		}


		// SAFETY: head will always point to something unless head and tail are equal.
		// the value within the buffer will be discarded, so ownership can be acquired.
		Some(unsafe { self.buffer[self.head].assume_init_read() })
	}

	/// Pop a value from the tail of the buffer.
	/// Returns [None] if the buffer is empty.
	pub fn pop_bottom(&mut self) -> Option<T> {
		if self.is_empty() {
			return None;
		}

		// SAFETY: tail will always point to something unless head and tail are equal.
		// the value within the buffer will be discarded, so ownership can be acquired.
		let val = unsafe { self.buffer[self.tail].assume_init_read() };

		if self.tail + 1 >= self.buffer.len() {
			self.tail = 0;
		} else {
			self.tail += 1;
		}

		
		Some(val)
	}

	/// Check if the buffer is empty.
	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.head == self.tail
	}

	/// Check if the buffer is full.
	#[inline(always)]
	pub fn is_full(&self) -> bool {
		let next = self.head + 1;

		if next == self.buffer.len() {
			// wrap around
			return self.tail == 0;
		}

		next == self.tail
	}
}


#[macro_export]
macro_rules! impl_tuple {
	($mac:tt) => {
		// i still hate this
		$mac!(A);
		$mac!(A, B);
		$mac!(A, B, C);
		$mac!(A, B, C, D);
		$mac!(A, B, C, D, E);
		$mac!(A, B, C, D, E, F);
		$mac!(A, B, C, D, E, F, G);
		$mac!(A, B, C, D, E, F, G, H);
		$mac!(A, B, C, D, E, F, G, H, I);
		$mac!(A, B, C, D, E, F, G, H, I, J);
		$mac!(A, B, C, D, E, F, G, H, I, J, K);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
		$mac!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
	};
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ring_buffer() {
		let mut buf = RingBuffer::new(4);

		assert_eq!(buf.pop_top(), None);
		assert_eq!(buf.pop_bottom(), None);
		assert_eq!(buf.is_empty(), true);
		assert_eq!(buf.is_full(), false);


		buf.push(3);
		// T
		//   H
		// 3 - - -

		assert_eq!(buf.is_empty(), false);
		assert_eq!(buf.is_full(), false);


		assert_eq!(buf.pop_top(), Some(3));
		// T
		// H
		// - - - -

		assert_eq!(buf.head, 0);
		assert_eq!(buf.tail, 0);
		assert_eq!(buf.is_empty(), true);
		assert_eq!(buf.is_full(), false);


		buf.push(4);
		// T
		//   H
		// 4 - - -

		assert_eq!(buf.pop_bottom(), Some(4));
		//   T
		//   H
		// - - - -

		assert_eq!(buf.head, 1);
		assert_eq!(buf.tail, 1);
		assert_eq!(buf.is_empty(), true);
		assert_eq!(buf.is_full(), false);


		buf.push(5);
		buf.push(6);
		buf.push(7);
		//   T
		// H
		// - 5 6 7

		assert_eq!(buf.head, 0);
		assert_eq!(buf.tail, 1);
		assert_eq!(buf.is_empty(), false);
		assert_eq!(buf.is_full(), true); // buffer is now full


		buf.push(8);
		//     T
		//   H
		// 8 - 5 6 7

		// buffer was full, time to grow
		assert_eq!(buf.head, 1);
		assert_eq!(buf.tail, 2);


		assert_eq!(buf.pop_bottom(), Some(5));
		assert_eq!(buf.pop_bottom(), Some(6));
		assert_eq!(buf.pop_bottom(), Some(7));
		assert_eq!(buf.pop_bottom(), Some(8));
		//   T
		//   H
		// - - - - -

		assert_eq!(buf.head, 1);
		assert_eq!(buf.tail, 1);
		assert_eq!(buf.is_empty(), true);
		assert_eq!(buf.is_full(), false);


		assert_eq!(buf.pop_bottom(), None);
		assert_eq!(buf.pop_top(), None);

		assert_eq!(buf.head, 1);
		assert_eq!(buf.tail, 1);



		buf.push(0);
		buf.push(1);
		buf.push(2);
		buf.push(3);
		buf.push(4);
		buf.push(5);
		//       T
		//     H
		// 4 5 - 0 1 2 3


		assert_eq!(buf.head, 2);
		assert_eq!(buf.tail, 3);
		assert_eq!(buf.is_empty(), false);
		assert_eq!(buf.is_full(), true);

		assert_eq!(buf.pop_bottom(), Some(0));
		assert_eq!(buf.pop_bottom(), Some(1));
		assert_eq!(buf.pop_bottom(), Some(2));
		assert_eq!(buf.pop_bottom(), Some(3));
		assert_eq!(buf.pop_bottom(), Some(4));
		assert_eq!(buf.pop_bottom(), Some(5));
		//     T
		//     H
		// - - - - - - -

		assert_eq!(buf.head, 2);
		assert_eq!(buf.tail, 2);
		assert_eq!(buf.is_empty(), true);
		assert_eq!(buf.is_full(), false);
	}
}
