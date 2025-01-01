
use crate::event::*;

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

    pub fn decode_head(&mut self) -> Result<Head<'a>, ()> {
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
	    Ok(Head { initial_byte, following_bytes })
	} else {
	    self.failed = true;
	    Err(())
	}
    }

    pub fn decode_bytes(&mut self, count: usize) -> Result<&'a [u8], ()> {
	if self.failed || self.data.len() < count {
	    self.failed = true;
	    return Err(());
	}

	let bytes = &self.data[0..count];
	self.data = &self.data[count..];

	Ok(bytes)
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_head() {
	let mut dec = Decoder::new(&[
	    0x0C,
	    0xF8, 0xDB,
	    0x99, 0x78, 0x14,
	    0x3A, 0x14, 0xE3, 0x17, 0x19,
	    0xBB, 0x9E, 0x1E, 0x5F, 0xD7, 0xE3, 0xA4, 0x07, 0xE1
	]);

	if let Ok(head) = dec.decode_head() {
	    assert_eq!(head.initial_byte, 0x0C);
	    assert_eq!(head.following_bytes, []);
	} else {
	    assert!(false);
	}

	if let Ok(head) = dec.decode_head() {
	    assert_eq!(head.initial_byte, 0xF8);
	    assert_eq!(head.following_bytes, [0xDB]);
	} else {
	    assert!(false);
	}

	if let Ok(head) = dec.decode_head() {
	    assert_eq!(head.initial_byte, 0x99);
	    assert_eq!(head.following_bytes, [0x78, 0x14]);
	} else {
	    assert!(false);
	}

	if let Ok(head) = dec.decode_head() {
	    assert_eq!(head.initial_byte, 0x3A);
	    assert_eq!(head.following_bytes, [0x14, 0xE3, 0x17, 0x19]);
	} else {
	    assert!(false);
	}
	
	if let Ok(head) = dec.decode_head() {
	    assert_eq!(head.initial_byte, 0xBB);
	    assert_eq!(head.following_bytes, [0x9E, 0x1E, 0x5F, 0xD7, 0xE3, 0xA4, 0x07, 0xE1]);
	} else {
	    assert!(false);
	}
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

	if let Ok(bytes) = dec.decode_bytes(3) {
	    assert_eq!(bytes, [0x84, 0xD8, 0xFF]);
	} else {
	    assert!(false);
	}

	assert!(dec.decode_bytes(2).is_err());
    }
    
}
