
use std::io::{Error, ErrorKind, Result, Write};

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
    
    fn encode_head_with_argument(&mut self, major_type: u8, argument: u64) -> Result<()> {
	if argument < 24 {
	    write_u8(&mut self.writer, major_type | (argument as u8))
	} else if argument <= 0xFF {
	    self.writer.write_all(&[
		major_type | 0x18,
		argument as u8
	    ])
	} else if argument <= 0xFFFF {
	    self.writer.write_all(&[
		major_type | 0x19,
		(argument >> 8) as u8,
		argument as u8
	    ])
	} else if argument <= 0xFFFF_FFFF {
	    self.writer.write_all(&[
		major_type | 0x1A,
		(argument >> 24) as u8,
		(argument >> 16) as u8,
		(argument >> 8) as u8,
		argument as u8
	    ])
	} else {
	    self.writer.write_all(&[
		major_type | 0x1B,
		(argument >> 56) as u8,
		(argument >> 48) as u8,
		(argument >> 40) as u8,
		(argument >> 32) as u8,
		(argument >> 24) as u8,
		(argument >> 16) as u8,
		(argument >> 8) as u8,
		argument as u8
	    ])
	}
    }
    
    fn encode_bytes(&mut self, bytes: &[u8]) -> Result<()> {
	self.writer.write_all(bytes)
    }

    pub fn encode_event<'a>(&mut self, event: &Event<'a>) -> Result<()> {
	use Event::*;
	match event {
	    UnsignedInteger(val) => self.encode_head_with_argument(0x00, *val),
	    NegativeInteger(val) => self.encode_head_with_argument(0x20, *val),
	    ByteString(content) => if let Ok(len) = u64::try_from(content.len()) {
		self.encode_head_with_argument(0x40, len)?;
		self.encode_bytes(content)
	    } else {
		Err(Error::from(ErrorKind::Other))
	    },
	    TextString(content) => if let Ok(len) = u64::try_from(content.len()) {
		self.encode_head_with_argument(0x60, len)?;
		self.encode_bytes(content)
	    } else {
		Err(Error::from(ErrorKind::Other))
	    },
	    Array(len) => self.encode_head_with_argument(0x80, *len),
	    Map(len) => self.encode_head_with_argument(0xA0, *len),
	    IndefiniteByteString => write_u8(&mut self.writer, 0x5F),
	    IndefiniteTextString => write_u8(&mut self.writer, 0x7F),
	    IndefiniteArray => write_u8(&mut self.writer, 0x9F),
	    IndefiniteMap => write_u8(&mut self.writer, 0xBF),
	    Tag(val) => self.encode_head_with_argument(0xC0, *val),
	    Simple(val) => if 24 <= *val && *val <= 31 {
		Err(Error::from(ErrorKind::Other))
	    } else {
		self.encode_head_with_argument(0xE0, *val as u64)
	    },
	    Float(val) => {
		let ai = match val.len() {
		    1 => 0x18,
		    2 => 0x19,
		    4 => 0x1A,
		    8 => 0x1B,
		    _ => {
			return Err(Error::from(ErrorKind::Other));
		    }
		};
		write_u8(&mut self.writer, 0xE0 | ai)?;
		self.encode_bytes(val)
	    },
	    Break => write_u8(&mut self.writer, 0xFF),
	    End => Ok(())
	}
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_head_with_argument() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_head_with_argument(0x20, 23);
	let _ = enc.encode_head_with_argument(0xA0, 0x6B);
	let _ = enc.encode_head_with_argument(0x40, 0x6A35);
	let _ = enc.encode_head_with_argument(0x60, 0x0614_82FA);
	let _ = enc.encode_head_with_argument(0x80, 0xDEC9_E143_001A_BA53);

	assert_eq!(buf, [
	    0x37,
	    0xB8, 0x6B,
	    0x59, 0x6A, 0x35,
	    0x7A, 0x06, 0x14, 0x82, 0xFA,
	    0x9B, 0xDE, 0xC9, 0xE1, 0x43, 0x00, 0x1A, 0xBA, 0x53
	]);
    }

    #[test]
    fn test_encode_bytes() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_bytes(&[0x01, 0x1E, 0x53]);
	let _ = enc.encode_bytes(&[0x7C]);

	assert_eq!(buf, [0x01, 0x1E, 0x53, 0x7C]);
    }

    #[test]
    fn test_encode_event_integer() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_event(&Event::UnsignedInteger(0x04));
	let _ = enc.encode_event(&Event::UnsignedInteger(0xA1));
	let _ = enc.encode_event(&Event::UnsignedInteger(0x087B));
	let _ = enc.encode_event(&Event::NegativeInteger(0x4CEB_716E));
	let _ = enc.encode_event(&Event::NegativeInteger(0xC1C0_067D_BA82_C53F));

	assert_eq!(buf, [
	    0x04,
	    0x18, 0xA1,
	    0x19, 0x08, 0x7B,
	    0x3A, 0x4C, 0xEB, 0x71, 0x6E,
	    0x3B, 0xC1, 0xC0, 0x06, 0x7D, 0xBA, 0x82, 0xC5, 0x3F
	]);
    }

    #[test]
    fn test_encode_event_string() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_event(&Event::ByteString(&[0x3C, 0x6A]));
	let _ = enc.encode_event(&Event::TextString(&[0x61, 0x62, 0x63]));
	let _ = enc.encode_event(&Event::IndefiniteByteString);
	let _ = enc.encode_event(&Event::IndefiniteTextString);

	assert_eq!(buf, [
	    0x42, 0x3C, 0x6A,
	    0x63, 0x61, 0x62, 0x63,
	    0x5F,
	    0x7F
	]);
    }

    #[test]
    fn test_encode_event_array_map() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_event(&Event::Array(0xAC));
	let _ = enc.encode_event(&Event::Map(0x09));
	let _ = enc.encode_event(&Event::IndefiniteArray);
	let _ = enc.encode_event(&Event::IndefiniteMap);

	assert_eq!(buf, [
	    0x98, 0xAC,
	    0xA9,
	    0x9F,
	    0xBF
	]);
    }

    #[test]
    fn test_encode_event_tag() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_event(&Event::Tag(0x37A5));

	assert_eq!(buf, [
	    0xD9, 0x37, 0xA5
	]);
    }

    #[test]
    fn test_encode_event_simple() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_event(&Event::Simple(10));
	let _ = enc.encode_event(&Event::Simple(0x5C));

	assert_eq!(buf, [
	    0xEA,
	    0xF8, 0x5C
	]);
    }

    #[test]
    fn test_encode_event_float() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_event(&Event::Float(&[0xFC, 0x00]));
	let _ = enc.encode_event(&Event::Float(&[0xFF, 0x80, 0x00, 0x00]));
	let _ = enc.encode_event(&Event::Float(&[0xFF, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]));

	assert_eq!(buf, [
	    0xF9, 0xFC, 0x00,
	    0xFA, 0xFF, 0x80, 0x00, 0x00,
	    0xFB, 0xFF, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
	]);
    }

    #[test]
    fn test_encode_event_break() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_event(&Event::Break);

	assert_eq!(buf, [0xFF]);
    }

    #[test]
    fn test_encode_event_end() {
	let mut buf = Vec::<u8>::new();
	let mut enc = Encoder::new(&mut buf);

	let _ = enc.encode_event(&Event::End);

	assert_eq!(buf, []);
    }
    
}
