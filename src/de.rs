// use std::ops::{AddAssign, MulAssign, Neg};

// use serde::Deserialize;
// use serde::de::{
//     self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
//     VariantAccess, Visitor,
// };

// use error::{Error, Result};

// pub struct Deserializer<'de> {
//     // This string starts with the input data and characters are truncated off
//     // the beginning as data is parsed.
//     input: &'de str,
// }

// impl<'de> Deserializer<'de> {
//     // By convention, `Deserializer` constructors are named like `from_xyz`.
//     // That way basic use cases are satisfied by something like
//     // `serde_json::from_str(...)` while advanced use cases that require a
//     // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
//     pub fn from_str(input: &'de str) -> Self {
//         Deserializer { input }
//     }
// }

// // By convention, the public API of a Serde deserializer is one or more
// // `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// // depending on what Rust types the deserializer is able to consume as input.
// //
// // This basic deserializer supports only `from_str`.
// pub fn from_str<'a, T>(s: &'a str) -> Result<T>
// where
//     T: Deserialize<'a>,
// {
//     let mut deserializer = Deserializer::from_str(s);
//     let t = T::deserialize(&mut deserializer)?;
//     if deserializer.input.is_empty() {
//         Ok(t)
//     } else {
//         Err(Error::TrailingCharacters)
//     }
// }

// // SERDE IS NOT A PARSING LIBRARY. This impl block defines a few basic parsing
// // functions from scratch. More complicated formats may wish to use a dedicated
// // parsing library to help implement their Serde deserializer.
// impl<'de> Deserializer<'de> {
//     // Look at the first character in the input without consuming it.
//     fn peek_char(&mut self) -> Result<char> {
//         self.input.chars().next().ok_or(Error::Eof)
//     }

//     // Consume the first character in the input.
//     fn next_char(&mut self) -> Result<char> {
//         let ch = self.peek_char()?;
//         self.input = &self.input[ch.len_utf8()..];
//         Ok(ch)
//     }

//     // Parse the JSON identifier `true` or `false`.
//     fn parse_bool(&mut self) -> Result<bool> {
//         if self.input.starts_with("true") {
//             self.input = &self.input["true".len()..];
//             Ok(true)
//         } else if self.input.starts_with("false") {
//             self.input = &self.input["false".len()..];
//             Ok(false)
//         } else {
//             Err(Error::ExpectedBoolean)
//         }
//     }

//     // Parse a group of decimal digits as an unsigned integer of type T.
//     //
//     // This implementation is a bit too lenient, for example `001` is not
//     // allowed in JSON. Also the various arithmetic operations can overflow and
//     // panic or return bogus data. But it is good enough for example code!
//     fn parse_unsigned<T>(&mut self) -> Result<T>
//     where
//         T: AddAssign<T> + MulAssign<T> + From<u8>,
//     {
//         let mut int = match self.next_char()? {
//             ch @ '0'..='9' => T::from(ch as u8 - b'0'),
//             _ => {
//                 return Err(Error::ExpectedInteger);
//             }
//         };
//         loop {
//             match self.input.chars().next() {
//                 Some(ch @ '0'..='9') => {
//                     self.input = &self.input[1..];
//                     int *= T::from(10);
//                     int += T::from(ch as u8 - b'0');
//                 }
//                 _ => {
//                     return Ok(int);
//                 }
//             }
//         }
//     }

//     // Parse a possible minus sign followed by a group of decimal digits as a
//     // signed integer of type T.
//     fn parse_signed<T>(&mut self) -> Result<T>
//     where
//         T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
//     {
//         // Optional minus sign, delegate to `parse_unsigned`, negate if negative.
//         unimplemented!()
//     }

//     // Parse a string until the next '"' character.
//     //
//     // Makes no attempt to handle escape sequences. What did you expect? This is
//     // example code!
//     fn parse_string(&mut self) -> Result<&'de str> {
//         if self.next_char()? != '"' {
//             return Err(Error::ExpectedString);
//         }
//         match self.input.find('"') {
//             Some(len) => {
//                 let s = &self.input[..len];
//                 self.input = &self.input[len + 1..];
//                 Ok(s)
//             }
//             None => Err(Error::Eof),
//         }
//     }
// }

// impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
//     type Error = Error;

//     // Look at the input data to decide what Serde data model type to
//     // deserialize as. Not all data formats are able to support this operation.
//     // Formats that support `deserialize_any` are known as self-describing.
//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         match self.peek_char()? {
//             'n' => self.deserialize_unit(visitor),
//             't' | 'f' => self.deserialize_bool(visitor),
//             '"' => self.deserialize_str(visitor),
//             '0'..='9' => self.deserialize_u64(visitor),
//             '-' => self.deserialize_i64(visitor),
//             '[' => self.deserialize_seq(visitor),
//             '{' => self.deserialize_map(visitor),
//             _ => Err(Error::Syntax),
//         }
//     }

//     // Uses the `parse_bool` parsing function defined above to read the JSON
//     // identifier `true` or `false` from the input.
//     //
//     // Parsing refers to looking at the input and deciding that it contains the
//     // JSON value `true` or `false`.
//     //
//     // Deserialization refers to mapping that JSON value into Serde's data
//     // model by invoking one of the `Visitor` methods. In the case of JSON and
//     // bool that mapping is straightforward so the distinction may seem silly,
//     // but in other cases Deserializers sometimes perform non-obvious mappings.
//     // For example the TOML format has a Datetime type and Serde's data model
//     // does not. In the `toml` crate, a Datetime in the input is deserialized by
//     // mapping it to a Serde data model "struct" type with a special name and a
//     // single field containing the Datetime represented as a string.
//     fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         Error::UnsupportedType
//     }

//     // The `parse_signed` function is generic over the integer type `T` so here
//     // it is invoked with `T=i8`. The next 8 methods are similar.
//     fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_i8(self.parse_signed()?)
//     }

//     fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_i16(self.parse_signed()?)
//     }

//     fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_i32(self.parse_signed()?)
//     }

//     fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_i64(self.parse_signed()?)
//     }

//     fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_u8(self.parse_unsigned()?)
//     }

//     fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_u16(self.parse_unsigned()?)
//     }

//     fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_u32(self.parse_unsigned()?)
//     }

//     fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_u64(self.parse_unsigned()?)
//     }

//     // Float parsing is stupidly hard.
//     fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         unimplemented!()
//     }

//     // Float parsing is stupidly hard.
//     fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         unimplemented!()
//     }

//     // The `Serializer` implementation on the previous page serialized chars as
//     // single-character strings so handle that representation here.
//     fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         // Parse a string, check that it is one character, call `visit_char`.
//         unimplemented!()
//     }

//     // Refer to the "Understanding deserializer lifetimes" page for information
//     // about the three deserialization flavors of strings in Serde.
//     fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         visitor.visit_borrowed_str(self.parse_string()?)
//     }

//     fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         self.deserialize_str(visitor)
//     }

//     // The `Serializer` implementation on the previous page serialized byte
//     // arrays as JSON arrays of bytes. Handle that representation here.
//     fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         unimplemented!()
//     }

//     fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         unimplemented!()
//     }

//     // An absent optional is represented as the JSON `null` and a present
//     // optional is represented as just the contained value.
//     //
//     // As commented in `Serializer` implementation, this is a lossy
//     // representation. For example the values `Some(())` and `None` both
//     // serialize as just `null`. Unfortunately this is typically what people
//     // expect when working with JSON. Other formats are encouraged to behave
//     // more intelligently if possible.
//     fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         Error::UnsupportedType
//     }

//     // In Serde, unit means an anonymous value containing no data.
//     fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         Error::UnsupportedType
//     }

//     // Unit struct means a named value containing no data.
//     fn deserialize_unit_struct<V>(
//         self,
//         _name: &'static str,
//         visitor: V,
//     ) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         Error::UnsupportedType
//     }

//     // As is done here, serializers are encouraged to treat newtype structs as
//     // insignificant wrappers around the data they contain. That means not
//     // parsing anything other than the contained value.
//     fn deserialize_newtype_struct<V>(
//         self,
//         _name: &'static str,
//         visitor: V,
//     ) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         Err(Error::UnsupportedType)
//     }

//     // Deserialization of compound types like sequences and maps happens by
//     // passing the visitor an "Access" object that gives it the ability to
//     // iterate through the data contained in the sequence.
//     fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         // Parse the opening bracket of the sequence.
//         if self.next_char()? == 'l' {
//             // Give the visitor access to each element of the sequence.
//             let value = visitor.visit_seq(CommaSeparated::new(&mut self))?;
//             // Parse the closing bracket of the sequence.
//             if self.next_char()? == 'e' {
//                 Ok(value)
//             } else {
//                 Err(Error::ExpectedArrayEnd)
//             }
//         } else {
//             Err(Error::ExpectedArray)
//         }
//     }

//     // Tuples look just like sequences in JSON. Some formats may be able to
//     // represent tuples more efficiently.
//     //
//     // As indicated by the length parameter, the `Deserialize` implementation
//     // for a tuple in the Serde data model is required to know the length of the
//     // tuple before even looking at the input data.
//     fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         Err(Error::UnsupportedType)
//     }

//     // Tuple structs look just like sequences in JSON.
//     fn deserialize_tuple_struct<V>(
//         self,
//         _name: &'static str,
//         _len: usize,
//         visitor: V,
//     ) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         Err(Error::UnsupportedType)
//     }

//     // Much like `deserialize_seq` but calls the visitors `visit_map` method
//     // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
//     // method with a `SeqAccess` implementation.
//     fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         // Parse the opening brace of the map.
//         if self.next_char()? == 'd' {
//             // Give the visitor access to each entry of the map.
//             let value = visitor.visit_map(CommaSeparated::new(&mut self))?;
//             // Parse the closing brace of the map.
//             if self.next_char()? == 'e' {
//                 Ok(value)
//             } else {
//                 Err(Error::ExpectedMapEnd)
//             }
//         } else {
//             Err(Error::ExpectedMap)
//         }
//     }

//     // Structs look just like maps in JSON.
//     //
//     // Notice the `fields` parameter - a "struct" in the Serde data model means
//     // that the `Deserialize` implementation is required to know what the fields
//     // are before even looking at the input data. Any key-value pairing in which
//     // the fields cannot be known ahead of time is probably a map.
//     fn deserialize_struct<V>(
//         self,
//         _name: &'static str,
//         _fields: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         self.deserialize_map(visitor)
//     }

//     fn deserialize_enum<V>(
//         self,
//         _name: &'static str,
//         _variants: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         if self.peek_char()? == '"' {
//             // Visit a unit variant.
//             visitor.visit_enum(self.parse_string()?.into_deserializer())
//         } else if self.next_char()? == '{' {
//             // Visit a newtype variant, tuple variant, or struct variant.
//             let value = visitor.visit_enum(Enum::new(self))?;
//             // Parse the matching close brace.
//             if self.next_char()? == '}' {
//                 Ok(value)
//             } else {
//                 Err(Error::ExpectedMapEnd)
//             }
//         } else {
//             Err(Error::ExpectedEnum)
//         }
//     }

//     // An identifier in Serde is the type that identifies a field of a struct or
//     // the variant of an enum. In JSON, struct fields and enum variants are
//     // represented as strings. In other formats they may be represented as
//     // numeric indices.
//     fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         self.deserialize_str(visitor)
//     }

//     // Like `deserialize_any` but indicates to the `Deserializer` that it makes
//     // no difference which `Visitor` method is called because the data is
//     // ignored.
//     //
//     // Some deserializers are able to implement this more efficiently than
//     // `deserialize_any`, for example by rapidly skipping over matched
//     // delimiters without paying close attention to the data in between.
//     //
//     // Some formats are not able to implement this at all. Formats that can
//     // implement `deserialize_any` and `deserialize_ignored_any` are known as
//     // self-describing.
//     fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         self.deserialize_any(visitor)
//     }
// }
