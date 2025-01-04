
/// イベント型。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Event<'a> {
    /// 符号なし整数イベント。
    UnsignedInteger(u64),

    /// 負整数イベント。
    NegativeInteger(u64),

    /// バイト列イベント。
    ByteString(&'a [u8]),

    /// 文字列イベント。
    TextString(&'a [u8]),

    /// 配列イベント。パラメーターは配列長。
    Array(u64),

    /// 連想配列イベント。パラメーターは連想数。
    Map(u64),

    /// 不定長バイト列イベント。
    IndefiniteByteString,

    /// 不定長文字列イベント。
    IndefiniteTextString,

    /// 不定長配列イベント。
    IndefiniteArray,

    /// 不定長連想配列イベント。
    IndefiniteMap,

    /// タグイベント。
    Tag(u64),

    /// 単純値イベント。
    Simple(u8),

    /// 半精度浮動小数点数イベント。
    HalfFloat(&'a [u8; 2]),

    /// 単精度浮動小数点数イベント。
    SingleFloat(&'a [u8; 4]),

    /// 倍精度浮動小数点数イベント。
    DoubleFloat(&'a [u8; 8]),

    /// ブレイクイベント。
    Break,

    /// データの終端を表すイベント。
    End
}

impl<'a> Eq for Event<'a> {}
