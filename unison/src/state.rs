use crate::*;


pub struct State {
	pub(crate) arena: arena::Arena,
	pub event_state: EventState,
}

impl State {
	pub fn new() -> Self {
		let event_state = EventState::new();

		Self {
			arena: arena::Arena::new(),
			event_state,
		}
	}

	pub fn clear(&mut self) {
		self.arena.clear();
	}

	pub fn alloc<T>(&mut self, val: T) -> arena::Ref<T> where T: Copy + PartialEq {
		self.arena.alloc(val)
	}

	pub fn get<T>(&self, r: arena::Ref<T>) -> Option<&T> where T: Copy + PartialEq {
		self.arena.get(r)
	}

	pub fn set<T>(&mut self, r: arena::Ref<T>, val: T) -> Option<()> where T: Copy + PartialEq {
		let p = self.arena.get_mut(r)?;
		let old = *p;
		*p = val;

		if old != val {
			// value has changed, emit event
			self.emit_ref_changed(r);
		}

		Some(())
	}

	pub fn mutate_ref<T, F>(&mut self, r: arena::Ref<T>, op: F) -> Option<()> where
		T: Copy + PartialEq,
		F: FnOnce(&mut T),
	{
		let old = *self.get(r)?;
		// TODO: since the first call to get worked, the other will work as well
		// we could probably use get_unchecked on the Ref
		op(self.arena.get_mut(r)?);
		let new = *self.arena.get(r)?;

		if old != new {
			// value has changed, emit event
			self.emit_ref_changed(r);
		}

		Some(())
	}

	pub fn emit_ref_changed<T>(&mut self, _r: arena::Ref<T>) {
		todo!()
	}


	pub fn get_event_type<T: 'static>(&mut self, name: &'static str) -> EventType<T> {
		self.event_state.get_event_type(name)
	}

	pub fn emit<T: 'static>(&mut self, ty: EventType<T>, val: T) {
		self.event_state.emit(ty, val)
	}
}
