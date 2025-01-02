# cbor-m

中くらいの大きさの (Medium-sized) CBORデータを処理するためのライブラリ。

## 特徴

- イベント駆動なAPI
- ゼロコピー・ノーアロケーション

## イベント

固定長バイト列と固定長文字列以外のデータ項目についてはイベントはヘッドに一致します。固定長バイト列と固定長文字列についてはヘッドに加えてその内容を表すバイト列もイベントに含まれます。

例えば、次のCBORデータは３つのイベント `A1` `18 2A` `65 68 65 6C 6C 6F` から構成されます。

```
A1 18 2A 65 68 65 6C 6C 6F
```

## 例 (デコード)

```rust
use cbor_m::event::Event;
use cbor_m::decode::Decoder;

let cbor = &[0xA1, 0x18, 0x2A, 0x65, 0x68, 0x65, 0x6C, 0x6C, 0x6F];
let mut dec = Decoder::new(cbor);

assert_eq!(dec.decode_event(), Ok(Event::Map(1)));
assert_eq!(dec.decode_event(), Ok(Event::UnsignedInteger(42)));
assert_eq!(dec.decode_event(), Ok(Event::TextString(b"hello")));
assert_eq!(dec.decode_event(), Ok(Event::End));
```

## 例 (エンコード)

```rust
use cbor_m::event::Event;
use cbor_m::encode::Encoder;

let mut buf = Vec::new();
let mut enc = Encoder::new(&mut buf);

let _ = enc.encode_event(&Event::Map(1));
let _ = enc.encode_event(&Event::UnsignedInteger(42));
let _ = enc.encode_event(&Event::TextString(b"hello"));

assert_eq!(buf, [0xA1, 0x18, 0x2A, 0x65, 0x68, 0x65, 0x6C, 0x6C, 0x6F]);
```
