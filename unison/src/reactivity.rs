use crate::*;


/// Any value that can be lazily evaluated given a [State]
pub trait LazyValue {
	type Output;

	/// Evaluate with the given [State].
	/// 
	/// Might return [None] when some value could not be evaluted with the given [State].
	fn eval(&self, state: &State) -> Option<Self::Output>;

	/// Unsafely evaluate without [State], bypassing all safety checks.
	/// 
	/// Only use when:
	/// - You are sure that all used [arena::Ref]s are alive:
	/// - The [State] (s) has not been cleared in any way.
	/// - There is no exclusive reference to the [State] (s).
	unsafe fn eval_unchecked(&self) -> Self::Output;
}

/// A lazily evaluated binding.
/// 
/// The inputs (I) are a [LazyValue] or a tuple of [LazyValue]s.
/// 
/// When evaluated, the given inputs are evaluated first and then passed to the given closure.
pub struct Binding<F, I, O> where
	I: LazyValue,
	F: Fn(I::Output) -> O,
{
	inputs: I,
	func: F,
}

impl<F, I, O> Binding<F, I, O> where
	I: LazyValue,
	F: Fn(I::Output) -> O,
{
	/// Create a new [Binding].
	/// 
	/// Prefer using the [bind!] macro.
	/// 
	/// Example:
	/// ```rust
	/// use unison::*;
	/// 
	/// let mut state = State::new();
	/// 
	/// let a = state.alloc(12);
	/// let b = state.alloc(20);
	/// 
	/// // note that `a` and `b` are evaluated and then passed to `ev_a` and `ev_b`
	/// let x = Binding::new((a, b), |(ev_a, ev_b)| ev_a + ev_b);
	/// 
	/// let y = x.eval(&state).unwrap();
	/// assert_eq!(y, 32);
	/// ```
	pub fn new(inputs: I, func: F) -> Self {
		Self { inputs, func }
	}
}

impl<F, I, O> LazyValue for Binding<F, I, O> where
	I: LazyValue,
	F: Fn(I::Output) -> O,
{
	type Output = O;

	fn eval(&self, state: &State) -> Option<Self::Output> {
		Some((self.func)(self.inputs.eval(state)?))
	}

	unsafe fn eval_unchecked(&self) -> Self::Output {
		(self.func)(self.inputs.eval_unchecked())
	}
}



pub mod extra {
	use super::*;
	/// Helper function for macro [bind!] that returns an impl [LazyValue]
	/// instead of a [Binding] for better type readability in IDEs.
	pub fn bind<F, I, O>(inputs: I, func: F) -> impl LazyValue<Output = O> where
		I: LazyValue,
		F: Fn(I::Output) -> O,
	{
		Binding::new(inputs, func)
	}
}


/// Create a lazily evaluated [Binding].
/// 
/// Any [arena::Ref]s used in this macro will be copied into the resulting [Binding] and evaluated
/// before passing them to the expression.
/// 
/// Syntax:
/// `bind!( [Ref,]+ => [expr] )`
/// 
/// Example:
/// ```rust
/// use unison::*;
/// 
/// let mut state = State::new();
/// 
/// let a = state.alloc(12);
/// let b = state.alloc(20);
/// 
/// let x = bind!(a, b => a + b);
/// 
/// let y = x.eval(&state).unwrap();
/// assert_eq!(y, 32);
/// ```
#[macro_export]
macro_rules! bind {
	( $( $name:ident ),* => $e:expr ) => {
		{
			$crate::reactivity_extra::bind(( $( $name, )* ), (move |( $( $name, )* )| $e))
		}
	};
}


impl<T: Copy> LazyValue for arena::Ref<T> {
	type Output = T;

	fn eval(&self, state: &State) -> Option<Self::Output> {
		state.arena.get(*self).map(|v|*v)
	}

	unsafe fn eval_unchecked(&self) -> Self::Output {
		*self.get_unchecked()
	}
}

impl LazyValue for () {
	type Output = ();

	fn eval(&self, _: &State) -> Option<Self::Output> {
		Some(())
	}

	unsafe fn eval_unchecked(&self) -> Self::Output {
		()
	}
}

macro_rules! impl_tuple_lazy {
	( $( $name:ident ),* ) => {

		impl< $( $name: LazyValue ),* > LazyValue for ($($name,)*) {
			type Output = ( $( $name::Output, )* );

			fn eval(&self, state: &State) -> Option<Self::Output> {
				#[allow(non_snake_case)]
				let ( $( $name, )* ) = &self;

				Some((
					$(
						match $name.eval(state) {
							Some(v) => v,
							None => return None,
						},
					)*
				))
			}

			unsafe fn eval_unchecked(&self) -> Self::Output {
				#[allow(non_snake_case)]
				let ( $( $name, )* ) = &self;

				( $( $name.eval_unchecked(), )* )
			}
		}

	};
}

impl_tuple!(impl_tuple_lazy);


/// Helper type to store values or [LazyValue]s.
pub enum Value<T: Copy> {
	Const(T),
	Lazy(Box<dyn LazyValue<Output = T>>),
}

impl<T: Copy> Value<T> {
	pub fn new(val: T) -> Self {
		Self::Const(val)
	}

	pub fn eval(&self, state: &State) -> Option<T> {
		match self {
			Self::Const(v) => Some(*v),
			Self::Lazy(v) => v.eval(state),
		}
	}
}

pub trait IntoValue {
	type Output: Copy;

	fn into_value(self) -> Value<Self::Output>;
}

impl<F, I, O> IntoValue for Binding<F, I, O> where
	F: Fn(I::Output) -> O + 'static,
	I: LazyValue + 'static,
	O: Copy + 'static,
{
	type Output = O;

	fn into_value(self) -> Value<Self::Output> {
		Value::Lazy(Box::new(self))
	}
}

impl<T: Copy> IntoValue for T {
	type Output = T;

	fn into_value(self) -> Value<Self::Output> {
		Value::Const(self)
	}
}

impl<T: Copy> IntoValue for Box<dyn LazyValue<Output = T>> {
	type Output = T;

	fn into_value(self) -> Value<Self::Output> {
		Value::Lazy(self)
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_bind() {
		let mut state = State::new();

		{
			let a = state.arena.alloc(12);
			let b = state.arena.alloc(20);
			let c = state.arena.alloc(20);

			let binding = bind!(a, b, c => a + b + c);

			assert_eq!(binding.eval(&state).unwrap(), 52); // i am a genius
		}


		{
			let a = state.arena.alloc(14);

			let binding = bind!(a => a);

			assert_eq!(binding.eval(&state).unwrap(), 14);
		}


		{
			let some1 = state.arena.alloc(10);

			let somevar = 34;

			let binding = bind!(some1 => format!("{} yee {}", some1 + somevar, "yos"));

			assert_eq!(binding.eval(&state).unwrap(), String::from("44 yee yos"));
		}


		{
			let yote = state.arena.alloc(10);

			let binding1 = bind!(yote => yote * yote);

			assert_eq!(binding1.eval(&state).unwrap(), 100);

			let binding2 = bind!(binding1, yote => binding1 + yote);

			// TODO: should Binding be Copy ?

			assert_eq!(binding2.eval(&state).unwrap(), 110);
		}


		{
			let _ = bind!( => 12); // i mean, why not
		}
	}
}

