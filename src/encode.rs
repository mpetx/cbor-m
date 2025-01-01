
use std::io::{Result, Write};

use crate::event::*;

pub struct Encoder<W: Write> {
    writer: W
}

fn write_u8<W: Write>(writer: &mut W, byte: u8) -> Result<()> {
    writer.write_all(&[byte])
}

impl<W: Write> Encoder<W> {

    pub fn new(writer: W) -> Encoder<W> {
	Encoder { writer }
    }

    pub fn encode_head(&mut self, head: &Head) -> Result<()> {
	write_u8(&mut self.writer, head.initial_byte)?;
	self.writer.write_all(head.following_bytes)
    }

    pub fn encode_bytes(&mut self, bytes: &[u8]) -> Result<()> {
	self.writer.write_all(bytes)
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_head() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);
	
	let head = Head::new(0x04, &[]);
	let _ = enc.encode_head(&head);

	let head = Head::new(0x39, &[0xF9, 0x0D]);
	let _ = enc.encode_head(&head);

	assert_eq!(buf, [0x04, 0x39, 0xF9, 0x0D]);
    }

    #[test]
    fn test_encode_bytes() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_bytes(&[0x01, 0x1E, 0x53]);
	let _ = enc.encode_bytes(&[0x7C]);

	assert_eq!(buf, [0x01, 0x1E, 0x53, 0x7C]);
    }
    
}
