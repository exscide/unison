use crate::*;

use std::collections::HashMap;


pub struct EventState {
	event_names: HashMap<&'static str, usize>,
	event_types: Vec<std::any::TypeId>,
	event_buffer: misc::RingBuffer<Event>,
}

impl EventState {
	pub fn new() -> Self {
		Self {
			event_names: HashMap::with_capacity(64),
			event_types: Vec::with_capacity(64),
			event_buffer: misc::RingBuffer::new(64),
		}
	}

	pub fn get_event_type<T: 'static>(&mut self, name: &'static str) -> EventType<T> {
		let typeid = std::any::TypeId::of::<T>();
		if let Some(ty) = self.event_names.get(name) {
			if typeid != self.event_types[*ty] {
				panic!("wrong type"); // TODO
			}

			return EventType::new(*ty);
		}

		self.event_types.push(typeid);
		EventType::new(self.event_types.len() - 1)
	}

	pub fn emit<T: 'static>(&mut self, ty: EventType<T>, val: T) {
		self.event_buffer.push(Event::new(ty, val))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventType<T>(usize, std::marker::PhantomData<T>);

impl<T> EventType<T> {
	pub fn new(ty: usize) -> Self {
		Self(ty, std::marker::PhantomData)
	}
}

pub struct Event(usize, Box<dyn std::any::Any>);

impl Event {
	pub fn new<T: 'static>(ty: EventType<T>, val: T) -> Self {
		Self(ty.0, Box::new(val))
	}
}
