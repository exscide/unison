use crate::*;


pub trait Component {
	#![allow(unused_variables)]

	type Child: ComponentLike + container::Containable;

	fn build(&self, state: &mut State) -> Self::Child;
	fn draw<'a, B: Backend>(&self, state: &State, view: &mut B::View<'a>) {}
	fn layout(&self, state: &mut State) -> Layout { Layout::default() }
}


use paste::paste;

macro_rules! impl_get_set {
	($name:ident, $typ:ty) => {
		paste! {
			pub fn [<set_ $name>]<T: IntoValue<Output = $typ>>(&mut self, val: T) {
				self.$name = val.into_value()
			}

			pub fn [<get_ $name>](&self, state: &State) -> Option<$typ> {
				self.$name.eval(state)
			}
		}

	};
}


pub struct Layout {
	flex: Value<u32>,

	margin: Value<Bounds>,
	padding: Value<Bounds>,

	stack_orientation: Value<Orientation>,
	stack_spacing: Value<u32>,
}

impl Layout {
	pub fn new() -> Self {
		Self::default()
	}


	impl_get_set!(flex, u32);
	impl_get_set!(margin, Bounds);
	impl_get_set!(padding, Bounds);

	impl_get_set!(stack_orientation, Orientation);
	impl_get_set!(stack_spacing, u32);
}




impl Default for Layout {
	fn default() -> Self {
		Self {
			flex: 1.into_value(),

			margin: Bounds::new(0, 0, 0, 0).into_value(),
			padding: Bounds::new(0, 0, 0, 0).into_value(),

			stack_orientation: Orientation::default().into_value(),
			stack_spacing: 0.into_value(),
		}
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Orientation {
	Vertical,
	#[default]
	Horizontal,
}



pub trait ComponentLike {
	
}

impl<T: Component> ComponentLike for T {
	
}

impl ComponentLike for () {
	
}


macro_rules! impl_tuple_component {
	($($name:ident),*) => {
		impl< $($name: Component),* > ComponentLike for ($($name,)*) {

		}
	};
}

impl_tuple!(impl_tuple_component);

