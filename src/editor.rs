use crate::buffer::Buffer;

pub struct Editor<'a> {
	pub buffers: Vec<Buffer<'a>>,
}

impl<'a> Editor<'a> {
	pub fn get_reference(&self) -> Box<&Editor> {
		Box::new(self)
	}
}
