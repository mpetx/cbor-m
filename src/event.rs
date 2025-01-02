
#[derive(Clone, Copy, PartialEq, Debug)]
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

impl<'a> Eq for Event<'a> {}
