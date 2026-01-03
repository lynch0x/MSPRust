use std::{collections::BTreeMap, io::{Cursor, Read, Write}};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};


pub struct ASObject {
    pub name: Option<String>,
    pub items: BTreeMap<String, AMFValue>
}


pub enum AMFValue {
    INT(f64),
    BOOL(bool),
    STRING(String),
    ASObject(ASObject),
    ARRAY(Vec<AMFValue>),
    NULL,
    BYTEARRAY(Vec<u8>)
}

impl AMFValue {
    pub fn as_int(&self) -> f64 {
        match self {
            AMFValue::INT(v) => *v,
            _ => 0.0
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            AMFValue::BOOL(v) => *v,
            _ => false
        }
    }

    pub fn as_string(&self) -> &str {
        match self {
            AMFValue::STRING(v) => v,
            _ => ""
        }
    }

    pub fn as_object(&self) -> &ASObject {
        match self {
            AMFValue::ASObject(v) => v,
            _ => {
                static EMPTY: std::sync::OnceLock<ASObject> = std::sync::OnceLock::new();
                EMPTY.get_or_init(|| ASObject {
                    name: None,
                    items: BTreeMap::new(),
                })
            }
        }
    }

    pub fn as_array(&self) -> Option<&Vec<AMFValue>> {
        match self {
            AMFValue::ARRAY(v) => Some(v),
            _ => None
        }
    }

    pub fn as_bytearray(&self) -> &[u8] {
        match self {
            AMFValue::BYTEARRAY(v) => v,
            _ => &[]
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, AMFValue::NULL)
    }
}

pub struct AMFHeader {
    pub name: String,
    pub must_understand: bool,
    pub body: AMFValue
}

pub struct AMFBody {
    pub target: String,
    pub response: String,
    pub body: AMFValue
}

pub struct AMFMessage {
    pub headers: Vec<AMFHeader>,
    pub body: AMFBody,
    pub version: u16
}


pub enum AMFError {
    IoError,
    ValueNotFound,
    RequestFailed,
}

impl From<std::io::Error> for AMFError {
    fn from(err: std::io::Error) -> Self {
        AMFError::IoError
    }
}

impl From<std::string::FromUtf8Error> for AMFError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        AMFError::RequestFailed
    }
}

impl std::fmt::Display for AMFError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_static_str())
    }
}

impl AMFError {
    pub const fn as_static_str(&self) -> &'static str {
        match self {
            AMFError::IoError => "IO error",
            AMFError::ValueNotFound => "Value not found",
            AMFError::RequestFailed => "Request failed",
        }
    }
}



pub struct AMFDeserializer;

impl AMFDeserializer {
    pub fn deserialize_amf_message(input: Vec<u8>) -> Result<AMFValue, AMFError> {
        let mut cursor = Cursor::new(input);

        let _version = cursor.read_u16::<BigEndian>()?;

        let headers_count = cursor.read_u16::<BigEndian>()?;
        for _ in 0..headers_count {
            let _header_name = Self::read_string(&mut cursor)?;
            let _must_understand = cursor.read_u8()?;
            let _length = cursor.read_i32::<BigEndian>()?;
            let _header_body = Self::read_data(&mut cursor)?;
        }

        let _body_count = cursor.read_u16::<BigEndian>()?;
        let _target = Self::read_string(&mut cursor)?;
        let _response = Self::read_string(&mut cursor)?;
        let _length = cursor.read_i32::<BigEndian>()?;

        Self::read_data(&mut cursor)
    }

    fn read_data(cursor: &mut Cursor<Vec<u8>>) -> Result<AMFValue, AMFError> {
        let marker = cursor.read_u8()?;
        Self::read_data_with_marker(cursor, marker)
    }

    fn read_data_with_marker(cursor: &mut Cursor<Vec<u8>>, marker: u8) -> Result<AMFValue, AMFError> {
        match marker {
            0 => Ok(AMFValue::INT(cursor.read_f64::<BigEndian>()?)),
            1 => Ok(AMFValue::BOOL(cursor.read_u8()? != 0)),
            2 => Ok(AMFValue::STRING(Self::read_string(cursor)?)),
            3 => Ok(AMFValue::ASObject(Self::read_aso(cursor)?)),
            5 => Ok(AMFValue::NULL),
            8 => Ok(AMFValue::ASObject(Self::read_associative_array(cursor)?)),
            10 => Ok(AMFValue::ARRAY(Self::read_array(cursor)?)),
            11 => {
                Self::read_date_time(cursor)?;
                Ok(AMFValue::NULL)
            },
            16 => Ok(AMFValue::ASObject(Self::read_object(cursor)?)),
            _ => Err(AMFError::RequestFailed)
        }
    }

    fn read_object(cursor: &mut Cursor<Vec<u8>>) -> Result<ASObject, AMFError> {
        let name = Self::read_string(cursor)?;
        let mut aso = Self::read_aso(cursor)?;
        aso.name = Some(name);
        Ok(aso)
    }

    fn read_date_time(cursor: &mut Cursor<Vec<u8>>) -> Result<(), AMFError> {
        let pos = cursor.position();
        let len = cursor.get_ref().len() as u64;

        if pos + 10 > len {
            return Err(AMFError::RequestFailed);
        }

        cursor.set_position(pos + 10);
        Ok(())
    }

    fn read_array(cursor: &mut Cursor<Vec<u8>>) -> Result<Vec<AMFValue>, AMFError> {
        let length = cursor.read_i32::<BigEndian>()?;

        if length < 0 {
            return Err(AMFError::RequestFailed);
        }

        let mut result = Vec::with_capacity(length as usize);
        for _ in 0..length {
            result.push(Self::read_data(cursor)?);
        }
        Ok(result)
    }

    fn read_associative_array(cursor: &mut Cursor<Vec<u8>>) -> Result<ASObject, AMFError> {
        let pos = cursor.position();
        let len = cursor.get_ref().len() as u64;

        if pos + 4 > len {
            return Err(AMFError::RequestFailed);
        }

        cursor.set_position(pos + 4);
        Self::read_aso(cursor)
    }

    fn read_aso(cursor: &mut Cursor<Vec<u8>>) -> Result<ASObject, AMFError> {
        let mut map: BTreeMap<String, AMFValue> = BTreeMap::new();

        loop {
            let name = Self::read_string(cursor)?;
            let marker = cursor.read_u8()?;

            if marker == 9 {
                break;
            }

            let value = Self::read_data_with_marker(cursor, marker)?;
            map.insert(name, value);
        }

        Ok(ASObject {
            name: None,
            items: map
        })
    }

    fn read_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String, AMFError> {
        let len = cursor.read_u16::<BigEndian>()?;

        if len == 0 {
            return Ok(String::new());
        }

        let mut str_bytes = vec![0; len as usize];
        cursor.read_exact(&mut str_bytes)?;
        Ok(String::from_utf8(str_bytes)?)
    }
}

pub struct AMFSerializer;

impl AMFSerializer {
    pub fn serialize_amf_message(message: AMFMessage) -> Vec<u8> {
        let vec: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(vec);

        cursor.write_u16::<BigEndian>(0).unwrap();
        cursor.write_u16::<BigEndian>(message.headers.len() as u16).unwrap();

        for header in message.headers {
            Self::write_string(&mut cursor, &header.name);
            cursor.write_u8(header.must_understand as u8).unwrap();
            cursor.write_i32::<BigEndian>(-1).unwrap();
            Self::write_data(&mut cursor, header.body);
        }

        cursor.write_u16::<BigEndian>(1).unwrap();
        Self::write_string(&mut cursor, &message.body.target);
        Self::write_string(&mut cursor, &message.body.response);
        cursor.write_i32::<BigEndian>(-1).unwrap();
        Self::write_data(&mut cursor, message.body.body);

        cursor.into_inner()
    }

    fn write_data(cursor: &mut Cursor<Vec<u8>>, value: AMFValue) {
        match value {
            AMFValue::NULL => {
                cursor.write_u8(5).unwrap();
            },
            AMFValue::BOOL(value) => {
                cursor.write_u8(1).unwrap();
                cursor.write_u8(value as u8).unwrap();
            },
            AMFValue::INT(value) => {
                cursor.write_u8(0).unwrap();
                cursor.write_f64::<BigEndian>(value).unwrap();
            }
            AMFValue::STRING(value) => {
                cursor.write_u8(2).unwrap();
                Self::write_string(cursor, &value);
            },
            AMFValue::ARRAY(value) => {
                cursor.write_u8(10).unwrap();
                cursor.write_i32::<BigEndian>(value.len() as i32).unwrap();
                for item in value {
                    Self::write_data(cursor, item);
                }
            },
            AMFValue::ASObject(value) => {
                if let Some(name) = value.name {
                    cursor.write_u8(16).unwrap();
                    Self::write_string(cursor, &name);
                } else {
                    cursor.write_u8(3).unwrap();
                }
                for (key, val) in value.items {
                    Self::write_string(cursor, &key);
                    Self::write_data(cursor, val);
                }
                Self::write_end_markup(cursor);
            },
            AMFValue::BYTEARRAY(value) => {
                cursor.write_u8(17).unwrap();
                cursor.write_u8(12).unwrap();
                let len = (value.len() as u32) << 1 | 1;
                Self::write_amf3_uint32_data(cursor, len);
                cursor.write_all(&value).unwrap();
            }
        }
    }

    fn write_amf3_uint32_data(cursor: &mut Cursor<Vec<u8>>, value: u32) {
        let value = value & 536870911;

        if value < 128 {
            cursor.write_u8(value as u8).unwrap();
        } else if value < 16384 {
            cursor.write_u8((value >> 7 | 128) as u8).unwrap();
            cursor.write_u8((value & 127) as u8).unwrap();
        } else if value < 2097152 {
            cursor.write_u8((value >> 14 | 128) as u8).unwrap();
            cursor.write_u8((value >> 7 & 127 | 128) as u8).unwrap();
            cursor.write_u8((value & 127) as u8).unwrap();
        } else {
            cursor.write_u8((value >> 22 | 128) as u8).unwrap();
            cursor.write_u8((value >> 15 & 127 | 128) as u8).unwrap();
            cursor.write_u8((value >> 8 & 127 | 128) as u8).unwrap();
            cursor.write_u8((value & 255) as u8).unwrap();
        }
    }

    fn write_end_markup(cursor: &mut Cursor<Vec<u8>>) {
        cursor.write_all(&[0, 0, 9]).unwrap();
    }

    fn write_string(cursor: &mut Cursor<Vec<u8>>, value: &str) {
        cursor.write_u16::<BigEndian>(value.len() as u16).unwrap();
        cursor.write_all(value.as_bytes()).unwrap();
    }
}