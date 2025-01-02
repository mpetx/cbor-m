
use crate::event::*;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Head<'a> {
    initial_byte: u8,
    following_bytes: &'a [u8]
}

impl<'a> Head<'a> {

    const MAJOR_TYPE_MASK: u8 = 0xE0;
    const ADDITIONAL_INFORMATION_MASK: u8 = 0x1F;

    fn new(initial_byte: u8, following_bytes: &'a [u8]) -> Head<'a> {
	Head {
	    initial_byte,
	    following_bytes
	}
    }

    fn major_type(&self) -> u8 {
	self.initial_byte & Self::MAJOR_TYPE_MASK
    }

    fn additional_information(&self) -> u8 {
	self.initial_byte & Self::ADDITIONAL_INFORMATION_MASK
    }

    fn argument(&self) -> Option<u64> {
	if !self.is_sound() {
	    return None;
	}

	let ai = self.additional_information();

	match ai {
	    0..24 => Some(ai as u64),
	    31 => None,
	    _ => {
		let mut arg = 0_u64;

		for &b in self.following_bytes {
		    arg = (arg << 8) | (b as u64);
		}

		Some(arg)
	    }
	}
    }
    
    fn is_sound(&self) -> bool {
	match self.additional_information() {
	    0..24 => self.following_bytes.len() == 0,
	    24 => self.following_bytes.len() == 1,
	    25 => self.following_bytes.len() == 2,
	    26 => self.following_bytes.len() == 4,
	    27 => self.following_bytes.len() == 8,
	    28 | 29 | 30 => false,
	    31 => match self.major_type() {
		0x40 | 0x60 | 0x80 | 0xA0 | 0xE0 => true,
		0x00 | 0x20 | 0xC0 => false,
		_ => panic!("unreachable")
	    },
	    _ => panic!("unreachable")
	}
    }
    
}

impl<'a> Eq for Head<'a> {}

pub struct Decoder<'a> {
    data: &'a [u8],
    failed: bool
}

impl<'a> Decoder<'a> {

    pub fn new(data: &'a [u8]) -> Decoder<'a> {
	Decoder {
	    data,
	    failed: false
	}
    }

    fn decode_head(&mut self) -> Result<Head<'a>, ()> {
	if self.failed || self.data.is_empty() {
	    self.failed = true;
	    return Err(());
	}

	let initial_byte = self.data[0];
	self.data = &self.data[1..];

	let following_bytes_len = match initial_byte & Head::ADDITIONAL_INFORMATION_MASK {
	    0..24 | 31 => 0,
	    24 => 1,
	    25 => 2,
	    26 => 4,
	    27 => 8,
	    28 | 29 | 30 => {
		self.failed = true;
		return Err(());
	    },
	    _ => panic!("unreachable")
	};

	if self.data.len() >= following_bytes_len {
	    let following_bytes = &self.data[0..following_bytes_len];
	    self.data = &self.data[following_bytes_len..];
	    Ok(Head::new(initial_byte, following_bytes))
	} else {
	    self.failed = true;
	    Err(())
	}
    }

    fn decode_bytes(&mut self, count: usize) -> Result<&'a [u8], ()> {
	if self.failed || self.data.len() < count {
	    self.failed = true;
	    return Err(());
	}

	let bytes = &self.data[0..count];
	self.data = &self.data[count..];

	Ok(bytes)
    }

    pub fn decode_event(&mut self) -> Result<Event, ()> {
	if self.failed {
	    return Err(());
	}
	
	if self.data.is_empty() {
	    return Ok(Event::End);
	}

	let head = self.decode_head()?;

	match head.major_type() >> 5 {
	    0 => Ok(Event::UnsignedInteger(head.argument().unwrap())),
	    1 => Ok(Event::NegativeInteger(head.argument().unwrap())),
	    2 => if head.additional_information() == 31 {
		Ok(Event::IndefiniteByteString)
	    } else {
		let len = usize::try_from(head.argument().unwrap());

		if len.is_err() {
		    self.failed = true;
		    return Err(());
		}
		
		let content = self.decode_bytes(len.unwrap())?;
		Ok(Event::ByteString(content))
	    },
	    3 => if head.additional_information() == 31 {
		Ok(Event::IndefiniteTextString)
	    } else {
		let len = usize::try_from(head.argument().unwrap());

		if len.is_err() {
		    self.failed = true;
		    return Err(());
		}
		
		let content = self.decode_bytes(len.unwrap())?;
		Ok(Event::TextString(content))
	    },
	    4 => if head.additional_information() == 31 {
		Ok(Event::IndefiniteArray)
	    } else {
		Ok(Event::Array(head.argument().unwrap()))
	    },
	    5 => if head.additional_information() == 31 {
		Ok(Event::IndefiniteMap)
	    } else {
		Ok(Event::Map(head.argument().unwrap()))
	    },
	    6 => Ok(Event::Tag(head.argument().unwrap())),
	    7 => match head.additional_information() {
		0..24 => Ok(Event::Simple(head.argument().unwrap() as u8)),
		24 => {
		    let val = head.argument().unwrap();
		    if val < 32 {
			self.failed = true;
			Err(())
		    } else {
			Ok(Event::Simple(val as u8))
		    }
		},
		25 | 26 | 27 => Ok(Event::Float(head.following_bytes)),
		31 => Ok(Event::Break),
		_ => panic!("unreachable")
	    },
	    _ => panic!("unreachable")
	}
    }
    
}
    
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_head() {
	let head = Head::new(0x15, &[]);
	assert_eq!(head.initial_byte, 0x15);
	assert_eq!(head.major_type(), 0x00);
	assert_eq!(head.additional_information(), 0x15);
	assert_eq!(head.following_bytes, []);
	assert!(head.is_sound());

	let head = Head::new(0x58, &[0xFB]);
	assert_eq!(head.initial_byte, 0x58);
	assert_eq!(head.major_type(), 0x40);
	assert_eq!(head.additional_information(), 0x18);
	assert_eq!(head.following_bytes, [0xFB]);
	assert!(head.is_sound());
    }

    #[test]
    fn test_head_argument() {
	let head = Head::new(0x31, &[]);
	assert_eq!(head.argument(), Some(0x11));

	let head = Head::new(0x58, &[0xAB]);
	assert_eq!(head.argument(), Some(0xAB));

	let head = Head::new(0x79, &[0xA5, 0xE6]);
	assert_eq!(head.argument(), Some(0xA5E6));

	let head = Head::new(0x9A, &[0x98, 0x36, 0x96, 0x0D]);
	assert_eq!(head.argument(), Some(0x9836_960D));

	let head = Head::new(0xBB, &[0x53, 0x84, 0xC4, 0x60, 0xFD, 0xB0, 0x04, 0xC4]);
	assert_eq!(head.argument(), Some(0x5384_C460_FDB0_04C4));

	let head = Head::new(0xFF, &[]);
	assert_eq!(head.argument(), None);
    }

    #[test]
    fn test_decode_head() {
	let mut dec = Decoder::new(&[
	    0x0C,
	    0xF8, 0xDB,
	    0x99, 0x78, 0x14,
	    0x3A, 0x14, 0xE3, 0x17, 0x19,
	    0xBB, 0x9E, 0x1E, 0x5F, 0xD7, 0xE3, 0xA4, 0x07, 0xE1
	]);

	assert_eq!(dec.decode_head(), Ok(Head::new(0x0C, &[])));
	assert_eq!(dec.decode_head(), Ok(Head::new(0xF8, &[0xDB])));
	assert_eq!(dec.decode_head(), Ok(Head::new(0x99, &[0x78, 0x14])));
	assert_eq!(dec.decode_head(), Ok(Head::new(0x3A, &[0x14, 0xE3, 0x17, 0x19])));
	assert_eq!(dec.decode_head(), Ok(Head::new(0xBB, &[0x9E, 0x1E, 0x5F, 0xD7, 0xE3, 0xA4, 0x07, 0xE1])));
    }

    #[test]
    fn test_decode_head_err() {
	let mut dec = Decoder::new(&[]);
	assert!(dec.decode_head().is_err());

	let mut dec = Decoder::new(&[0x1C]);
	assert!(dec.decode_head().is_err());

	let mut dec = Decoder::new(&[0x5A, 0x00, 0x00, 0x00]);
	assert!(dec.decode_head().is_err());
    }

    #[test]
    fn test_decode_bytes() {
	let mut dec = Decoder::new(&[0x84, 0xD8, 0xFF, 0x70]);

	assert_eq!(dec.decode_bytes(3), Ok::<&[u8], ()>(&[0x84, 0xD8, 0xFF]));

	assert!(dec.decode_bytes(2).is_err());
    }

    #[test]
    fn test_decode_event_integer() {
	let mut dec = Decoder::new(&[
	    0x0B,
	    0x18, 0x8C,
	    0x39, 0xB9, 0x37
	]);

	assert_eq!(dec.decode_event(), Ok(Event::UnsignedInteger(0x0B)));
	assert_eq!(dec.decode_event(), Ok(Event::UnsignedInteger(0x8C)));
	assert_eq!(dec.decode_event(), Ok(Event::NegativeInteger(0xB937)));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }

    #[test]
    fn test_decode_event_string() {
	let mut dec = Decoder::new(&[
	    0x43, 0x9D, 0x1B, 0x22,
	    0x78, 0x01, 0x4E,
	    0x5F,
	    0x7F
	]);

	assert_eq!(dec.decode_event(), Ok(Event::ByteString(&[0x9D, 0x1B, 0x22])));
	assert_eq!(dec.decode_event(), Ok(Event::TextString(&[0x4E])));
	assert_eq!(dec.decode_event(), Ok(Event::IndefiniteByteString));
	assert_eq!(dec.decode_event(), Ok(Event::IndefiniteTextString));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }

    #[test]
    fn test_decode_event_array_map() {
	let mut dec = Decoder::new(&[
	    0x83, 0x9F, 0xAC, 0xBF
	]);

	assert_eq!(dec.decode_event(), Ok(Event::Array(0x03)));
	assert_eq!(dec.decode_event(), Ok(Event::IndefiniteArray));
	assert_eq!(dec.decode_event(), Ok(Event::Map(0x0C)));
	assert_eq!(dec.decode_event(), Ok(Event::IndefiniteMap));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }

    #[test]
    fn test_decode_event_tag() {
	let mut dec = Decoder::new(&[0xD9, 0x5E, 0xD2]);

	assert_eq!(dec.decode_event(), Ok(Event::Tag(0x5ED2)));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }

    #[test]
    fn test_decode_event_simple() {
	let mut dec = Decoder::new(&[0xE7, 0xF8, 0x5E]);

	assert_eq!(dec.decode_event(), Ok(Event::Simple(0x07)));
	assert_eq!(dec.decode_event(), Ok(Event::Simple(0x5E)));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }

    #[test]
    fn test_decode_event_simple_err() {
	let mut dec = Decoder::new(&[0x78, 20]);
	assert!(dec.decode_event().is_err());

	let mut dec = Decoder::new(&[0x78, 0xFF]);
	assert!(dec.decode_event().is_err());
    }

    #[test]
    fn test_decode_event_float() {
	let mut dec = Decoder::new(&[
	    0xF9, 0x7C, 0x00,
	    0xFA, 0x7F, 0x80, 0x00, 0x00,
	    0xFB, 0x7F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
	]);

	assert_eq!(dec.decode_event(), Ok(Event::Float(&[0x7C, 0x00])));
	assert_eq!(dec.decode_event(), Ok(Event::Float(&[0x7F, 0x80, 0x00, 0x00])));
	assert_eq!(dec.decode_event(), Ok(Event::Float(&[0x7F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }

    #[test]
    fn test_decode_event_break() {
	let mut dec = Decoder::new(&[0xFF]);

	assert_eq!(dec.decode_event(), Ok(Event::Break));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }
    
}
