
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
    
}
