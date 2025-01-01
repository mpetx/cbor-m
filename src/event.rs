
pub struct Head<'a> {
    pub initial_byte: u8,
    pub following_bytes: &'a [u8]
}

impl<'a> Head<'a> {

    pub const MAJOR_TYPE_MASK: u8 = 0xE0;
    pub const ADDITIONAL_INFORMATION_MASK: u8 = 0x1F;

    pub fn new(initial_byte: u8, following_bytes: &'a [u8]) -> Head<'a> {
	Head {
	    initial_byte,
	    following_bytes
	}
    }

    pub fn major_type(&self) -> u8 {
	self.initial_byte & Self::MAJOR_TYPE_MASK
    }

    pub fn additional_information(&self) -> u8 {
	self.initial_byte & Self::ADDITIONAL_INFORMATION_MASK
    }

    pub fn argument(&self) -> Option<u64> {
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
    
    pub fn is_sound(&self) -> bool {
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

pub enum Event<'a> {
    UnsignedInteger(u64),
    NegativeInteger(u64),
    ByteString(&'a [u8]),
    TextString(&'a [u8]),
    Array(u64),
    Map(u64),
    IndefiniteByteString,
    IndefiniteTextString,
    IndefiniteArray,
    IndefiniteMap,
    Tag(u64),
    Simple(u8),
    Float(&'a [u8]),
    Break,
    End
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
}
