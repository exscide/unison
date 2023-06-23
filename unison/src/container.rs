use crate::*;


/// A container that may or may not contain itself.
pub struct ComponentContainer<T: Component> {
	component: T,
	child: <T::Child as Containable>::Container,
	pub layout: Layout,
}

impl<T: Component> ComponentContainer<T> {
	pub fn new(component: T, state: &mut State) -> Self {
		let child = component.build(state);
		let layout = component.layout(state);

		Self {
			component,
			child: child.contain(state),
			layout,
		}
	}
}


/// For types that may or may not be a [Container].
pub trait ContainerLike {
	fn draw<'a, B: Backend>(&self, state: &State, parent_layout: &Layout, view: &mut B::View<'a>);
}

impl<T: Component> ContainerLike for ComponentContainer<T> {
	fn draw<'a, B: Backend>(&self, state: &State, _parent_layout: &Layout, view: &mut B::View<'a>) {
		view.apply_bounds(self.layout.get_margin(state).unwrap()); // TODO
		self.component.draw::<B>(state, view);

		view.apply_bounds(self.layout.get_padding(state).unwrap());
		self.child.draw::<B>(state, &self.layout, view);
	}
}

impl ContainerLike for () {
	fn draw<'a, B: Backend>(&self, _state: &State, _parent_layout: &Layout, _view: &mut B::View<'a>) {}
}


/// For types that may be contained within a [Container].
pub trait Containable {
	type Container: ContainerLike;

	fn contain(self, state: &mut State) -> Self::Container;
}

impl<T: Component> Containable for T where
	T::Child: Component,
{
	type Container = ComponentContainer<T>;

	fn contain(self, state: &mut State) -> Self::Container {
		ComponentContainer::new(self, state)
	}
}

impl Containable for () {
	type Container = ();

	fn contain(self, _: &mut State) -> Self::Container {
		self
	}
}


/// A tree of [Container]s.
pub struct ComponentTree<T: Component> {
	tree: ComponentContainer<T>,
	_pin: std::marker::PhantomPinned,
	tree_idx: usize,
}

impl<T: Component> ComponentTree<T> {
	pub fn new(root: T, state: &mut State) -> Self {
		static TREE_IDX: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

		let tree = Self {
			tree: ComponentContainer::new(root, state),
			_pin: std::marker::PhantomPinned,
			tree_idx: TREE_IDX.load(std::sync::atomic::Ordering::Relaxed)
		};

		TREE_IDX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

		tree
	}

	pub fn get_event_handler(&mut self, mut handler: EventHandlerRef) -> Option<&mut dyn EventHandler> {
		if handler.tree_idx != self.tree_idx {
			return None;
		}

		// SAFETY: all of the Containers within this ComponentTree that can be accessed cannot move or drop
		// as long as the tree ist alive and retains the same number
		Some(unsafe { handler.container.as_mut() })
	}

	pub fn draw<'a, B: Backend>(&self, state: &State, view: &mut B::View<'a>) {
		self.tree.draw::<B>(state, &Layout::new(), view);
	}
}

pub trait EventHandler {
	fn handle(&mut self, ev: Event);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventHandlerRef {
	container: std::ptr::NonNull<dyn EventHandler>,
	tree_idx: usize,
}


macro_rules! impl_tuple_container {
	($($name:ident),*) => {
		impl< $($name: Component),* > container::ContainerLike for ($(container::ComponentContainer< $name >,)*) {
			fn draw<'a, Ba: Backend>(&self, state: &State, parent_layout: &Layout, view: &mut Ba::View<'a>) {
				#![allow(unused_assignments)]

				#[allow(non_snake_case)]
				let ($($name,)*) = self;

				let mut count = 0;
				let mut spacers = 0;

				let mut counts = Vec::new();

				$(
					{
						let c = $name.layout.get_flex(state).unwrap();
						counts.push(c);
						count += c;
						spacers += 1;
					}
				)*

				spacers -= 1;

				let orient = parent_layout.get_stack_orientation(state).unwrap();
				let size = view.viewport_size();
				let size = match orient {
					Orientation::Horizontal => size.0,
					Orientation::Vertical => size.1,
				};

				let spacing = parent_layout.get_stack_spacing(state).unwrap();

				let mut offset = 0;

				let component_space = (size - spacers * spacing);
				let size_per_count = (component_space as f32 / count as f32);

				let mut cur_c = 0;

				$(
					{
						let c = counts[cur_c];
						let mut el_size = (size_per_count * c as f32) as u32;

						if cur_c == counts.len()-1 {
							el_size = size - offset;
						}

						view.push();
						match orient {
							Orientation::Horizontal => view.set_viewport_horizontal(offset, el_size),
							Orientation::Vertical => view.set_viewport_vertical(offset, el_size),
						}
						$name.draw::<Ba>(state, parent_layout, view);
						view.restore();

						cur_c += 1;
						offset += el_size + spacing;
					}
				)*
			}
		}

		impl< $($name: Component),* > container::Containable for ($($name,)*) {
			type Container = ($(container::ComponentContainer< $name >,)*);

			fn contain(self, state: &mut State) -> Self::Container {
				#[allow(non_snake_case)]
				let ($($name,)*) = self;

				($(container::ComponentContainer::new($name, state),)*)
			}
		}
	};
}

impl_tuple!(impl_tuple_container);

