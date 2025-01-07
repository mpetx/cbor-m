
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

/// デコーダー型。
pub struct Decoder<'a> {
    data: &'a [u8]
}

fn decode_head<'a>(data: &'a [u8]) -> Result<(Head<'a>, &'a [u8]), ()> {
    if data.is_empty() {
	panic!("INTERNAL ERROR: decode_head on an empty byte string.");
    }

    let ib = data[0];
    let rest = &data[1..];

    let ai = ib & Head::ADDITIONAL_INFORMATION_MASK;

    if ai == 28 || ai == 29 || ai == 30 {
	return Err(());
    }

    let bytes_len = match ai {
	0..24 | 31 => 0,
	24 => 1,
	25 => 2,
	26 => 4,
	27 => 8,
	_ => panic!("unreachable")
    };

    if rest.len() >= bytes_len {
	let bytes = &rest[0..bytes_len];
	let rest = &rest[bytes_len..];
	let head = Head::new(ib, bytes);
	Ok((head, rest))
    } else {
	Err(())
    }
}

fn decode_bytes<'a>(data: &'a [u8], count: usize) -> Result<(&'a [u8], &'a [u8]), ()> {
    if data.len() >= count {
	Ok((&data[0..count], &data[count..]))
    } else {
	Err(())
    }
}

fn decode_event<'a>(data: &'a [u8]) -> Result<(Event<'a>, &'a [u8]), ()> {
    if data.is_empty() {
	return Ok((Event::End, data));
    }

    let (head, rest) = decode_head(data)?;

    match head.major_type() >> 5 {
	0 => Ok((Event::UnsignedInteger(head.argument().unwrap()), rest)),
	1 => Ok((Event::NegativeInteger(head.argument().unwrap()), rest)),
	2 => if head.additional_information() == 31 {
	    Ok((Event::IndefiniteByteString, rest))
	} else if let Ok(len) = usize::try_from(head.argument().unwrap()) {
	    let (content, rest) = decode_bytes(rest, len)?;
	    Ok((Event::ByteString(content), rest))
	} else {
	    Err(())
	},
	3 => if head.additional_information() == 31 {
	    Ok((Event::IndefiniteTextString, rest))
	} else if let Ok(len) = usize::try_from(head.argument().unwrap()) {
	    let (content, rest) = decode_bytes(rest, len)?;
	    Ok((Event::TextString(content), rest))
	} else {
	    Err(())
	},
	4 => if head.additional_information() == 31 {
	    Ok((Event::IndefiniteArray, rest))
	} else {
	    Ok((Event::Array(head.argument().unwrap()), rest))
	},
	5 => if head.additional_information() == 31 {
	    Ok((Event::IndefiniteMap, rest))
	} else {
	    Ok((Event::Map(head.argument().unwrap()), rest))
	},
	6 => Ok((Event::Tag(head.argument().unwrap()), rest)),
	7 => match head.additional_information() {
	    0..24 => Ok((Event::Simple(head.additional_information()), rest)),
	    24 => {
		let val = head.argument().unwrap();
		if val < 32 {
		    Err(())
		} else {
		    Ok((Event::Simple(val as u8), rest))
		}
	    },
	    25 => Ok((Event::HalfFloat(head.following_bytes.try_into().unwrap()), rest)),
	    26 => Ok((Event::SingleFloat(head.following_bytes.try_into().unwrap()), rest)),
	    27 => Ok((Event::DoubleFloat(head.following_bytes.try_into().unwrap()), rest)),
	    31 => Ok((Event::Break, rest)),
	    _ => panic!("unreachable")
	},
	_ => panic!("unreachable")
    }
}

impl<'a> Decoder<'a> {

    /// デコーダーを作成する。パラメーターはデコード対象のバイト列。
    pub fn new(data: &'a [u8]) -> Decoder<'a> {
	Decoder { data }
    }

    /// 次のイベントを取得する。
    pub fn decode_event(&mut self) -> Result<Event, ()> {
	let (event, rest) = decode_event(self.data)?;

	self.data = rest;

	Ok(event)
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
	let bytes = &[0x0C, 0x6B];
	assert_eq!(decode_head(bytes), Ok((Head::new(0x0C, &bytes[1..1]), &bytes[1..])));

	let bytes = &[0xF8, 0xDB, 0x02, 0x35];
	assert_eq!(decode_head(bytes), Ok((Head::new(0xF8, &bytes[1..2]), &bytes[2..])));

	let bytes = &[0x99, 0x78, 0x14, 0xF4, 0xC6, 0xBE];
	assert_eq!(decode_head(bytes), Ok((Head::new(0x99, &bytes[1..3]), &bytes[3..])));

	let bytes = &[0x3A, 0x14, 0xE3, 0x17, 0x19, 0x49];
	assert_eq!(decode_head(bytes), Ok((Head::new(0x3A, &bytes[1..5]), &bytes[5..])));

	let bytes = &[0xBB, 0x9E, 0x1E, 0x5F, 0xD7, 0xE3, 0xA4, 0x07, 0xE1];
	assert_eq!(decode_head(bytes), Ok((Head::new(0xBB, &bytes[1..9]), &bytes[9..])));
		      
    }

    #[test]
    fn test_decode_head_err() {
	let bytes = &[0x1C];
	assert_eq!(decode_head(bytes), Err(()));

	let bytes = &[0x5A, 0x00, 0x00, 0x00];
	assert_eq!(decode_head(bytes), Err(()));
    }

    #[test]
    fn test_decode_bytes() {
	let bytes = &[0x84, 0xD8, 0xFF, 0x70];
	assert_eq!(decode_bytes(bytes, 3), Ok((&bytes[0..3], &bytes[3..])));

	let bytes = &[0x34, 0x1B];
	assert_eq!(decode_bytes(bytes, 3), Err(()));
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

	assert_eq!(dec.decode_event(), Ok(Event::HalfFloat(&[0x7C, 0x00])));
	assert_eq!(dec.decode_event(), Ok(Event::SingleFloat(&[0x7F, 0x80, 0x00, 0x00])));
	assert_eq!(dec.decode_event(), Ok(Event::DoubleFloat(&[0x7F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }

    #[test]
    fn test_decode_event_break() {
	let mut dec = Decoder::new(&[0xFF]);

	assert_eq!(dec.decode_event(), Ok(Event::Break));
	assert_eq!(dec.decode_event(), Ok(Event::End));
    }
    
}
