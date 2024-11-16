use std::{collections::HashMap,  io::{Cursor, Read, Seek, Write}};

#[derive(Debug)] 
pub struct ASObject{
    pub name:Option<String>,
    pub items:HashMap<String,AMFValue>
}
#[derive(Debug)] 
pub enum AMFValue{
    INT(f64),
    BOOL(bool),
    STRING(String),
    ASObject(ASObject),
    ARRAY(Vec<AMFValue>),
    NULL,
    BYTEARRAY(Vec<u8>)
}

pub struct  AMFHeader{
    pub name: String,
    pub must_understand: bool,
    pub body: AMFValue
}
pub struct AMFBody{
    pub target:String,
    pub response:String,
    pub body:AMFValue
}
pub struct AMFMessage{
    pub headers: Vec<AMFHeader>,
    pub body: AMFBody,
    pub version: u16
}
pub struct AMFDeserializer;
impl AMFDeserializer{
    pub fn deserialize_amf_message(input:Vec<u8>)->AMFValue{
        let mut cursor = Cursor::new(input);
        let _ = Self::read_u16(&mut cursor);
        let headers_count = Self::read_u16(&mut cursor);
        for n in 0..headers_count{
            let _ = Self::read_string(&mut cursor);
            let _ = Self::read_byte(&mut cursor);
            let _  = Self::read_i32(&mut cursor);
            let _ = Self::read_data(&mut cursor);
        }
        let _ = Self::read_u16(&mut cursor);
        let _ = Self::read_string(&mut cursor);
        let _ = Self::read_string(&mut cursor);
        let _ = Self::read_i32(&mut cursor);
        Self::read_data(&mut cursor)
    }
    fn read_data(cursor:&mut Cursor<Vec<u8>>)->AMFValue{
        let marker = Self::read_byte(cursor);
        Self::read_data_with_marker(cursor, marker)
    }
    fn read_data_with_marker(cursor:&mut Cursor<Vec<u8>>,marker:u8)->AMFValue{
        
        match marker{
            0=>AMFValue::INT(Self::read_f64(cursor)),
            1=>AMFValue::BOOL(Self::read_byte(cursor) != 0),
            2=>AMFValue::STRING(Self::read_string(cursor)),
            3=>AMFValue::ASObject(Self::read_aso(cursor)),
            5=>AMFValue::NULL,
            8=>AMFValue::ASObject(Self::read_associative_array(cursor)),
            10=>AMFValue::ARRAY(Self::read_array(cursor)),
            11=>{
                Self::read_date_time(cursor);
                AMFValue::NULL
            },
            16=>AMFValue::ASObject(Self::read_object(cursor)),
            _=>panic!("Unknown AMF0 type! Byte: {}",marker)
        }
    }
    fn read_object(cursor:&mut Cursor<Vec<u8>>)->ASObject{
        let name = Self::read_string(cursor);
        let mut aso:ASObject = Self::read_aso(cursor);
        aso.name = Some(name);
        aso
    }
    fn read_date_time(cursor:&mut Cursor<Vec<u8>>){
        cursor.set_position(cursor.position()+10);
    }
    fn read_array(cursor:&mut Cursor<Vec<u8>>)->Vec<AMFValue>{
        let length = Self::read_i32(cursor);
        (0..length).map(|_| Self::read_data(cursor)).collect()
    }
    fn read_associative_array(cursor:&mut Cursor<Vec<u8>>)->ASObject{
        cursor.set_position(cursor.position()+4);
        Self::read_aso(cursor)
    }
    fn read_aso(cursor:&mut Cursor<Vec<u8>>)->ASObject{
        let mut map:HashMap<String,AMFValue> = HashMap::new();
        loop{
            let name = Self::read_string(cursor);
            let marker = Self::read_byte(cursor);
            if marker == 9{
                break;
            }
            map.insert(name, Self::read_data_with_marker(cursor,marker));
        }
        ASObject{
            name:None,
            items:map
        }
    }
    fn read_i32(cursor:&mut Cursor<Vec<u8>>)->i32{
        let mut v:[u8;4] = [0;4];
        let _ = cursor.read_exact(&mut v);
        i32::from_be_bytes(v)
    }
    fn read_f64(cursor:&mut Cursor<Vec<u8>>)->f64{
        let mut v:[u8;8] = [0;8];
        let _ = cursor.read_exact(&mut v);
        f64::from_be_bytes(v)
    }
    fn read_byte(cursor:&mut Cursor<Vec<u8>>)->u8{
        let mut v:[u8;1] = [0;1];
        let _ = cursor.read_exact(&mut v);
        v[0]
    }
    fn read_string(cursor:&mut Cursor<Vec<u8>>)->String{
        let len = Self::read_u16(cursor);
        let mut str_bytes = vec![0;len as usize];
        let _ = cursor.read_exact(&mut str_bytes);
        String::from_utf8(str_bytes).unwrap()
    }
    fn read_u16(cursor:&mut Cursor<Vec<u8>>)->u16{
        let mut v:[u8;2] = [0;2];
        let _ = cursor.read_exact(&mut v);
        u16::from_be_bytes(v)
    }
}
pub struct AMFSerializer;
impl AMFSerializer{
    pub fn serialize_amf_message(message:AMFMessage)->Vec<u8>{
        let vec:Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(vec);
        Self::write_u16(&mut cursor, 0);
        Self::write_u16(&mut cursor, message.headers.len() as u16);
        for header in message.headers{
            Self::write_utf8(&mut cursor, header.name);
            Self::write_bool(&mut cursor, header.must_understand);
            Self::write_i32(&mut cursor, -1);
            Self::write_data(&mut cursor, header.body);
        }
        Self::write_u16(&mut cursor, 1);
        Self::write_utf8(&mut cursor, message.body.target);
        Self::write_utf8(&mut cursor, message.body.response);
        Self::write_i32(&mut cursor, -1);
        Self::write_data(&mut cursor, message.body.body);
        return cursor.get_ref().clone();
    }
    fn write_double(cursor:&mut Cursor<Vec<u8>>,value:f64){
        let _ = cursor.write_all(&value.to_be_bytes());
    }
    fn write_data(cursor:&mut Cursor<Vec<u8>>,value:AMFValue){
        match value{
            AMFValue::NULL=>{
                let _ = cursor.write_all(&[5 as u8]);
            },
            AMFValue::BOOL(value)=>{
                let _ = cursor.write_all(&[1 as u8]);
                Self::write_bool(cursor, value);
            },
            AMFValue::INT(value)=>{
                let _ = cursor.write_all(&[0 as u8]);
                Self::write_double(cursor, value);
            }
            AMFValue::STRING(value)=>{
                let _ = cursor.write_all(&[2 as u8]);
                Self::write_utf8(cursor, value);
            },
            AMFValue::ARRAY(value)=>{
                let _ = cursor.write_all(&[10 as u8]);
                Self::write_i32(cursor, value.len() as i32);
                for item in value{
                    Self::write_data(cursor, item);
                }
            },
            AMFValue::ASObject(value)=>{
                if let Some(name) = value.name{
                    let _ = cursor.write_all(&[16 as u8]);
                    Self::write_utf8(cursor, name);
                }else{
                    let _ = cursor.write_all(&[3 as u8]);
                }
               for item in value.items{
                    Self::write_utf8(cursor, item.0);
                    Self::write_data(cursor,item.1);
               }
               Self::write_end_markup(cursor);
            }
            _=>todo!()
        }
    }
    fn write_end_markup(cursor:&mut Cursor<Vec<u8>>){
        let _= cursor.write_all(&[0,0,9]);
    }
    fn write_u16(cursor:&mut Cursor<Vec<u8>>,value:u16){
        let _ =cursor.write_all(&value.to_be_bytes());
    }
    fn write_i32(cursor:&mut Cursor<Vec<u8>>,value:i32){
        let _ =cursor.write_all(&value.to_be_bytes());
    }
    fn write_utf8(cursor:&mut Cursor<Vec<u8>>,value:String){
        let len = value.len() as u16;
        let _ = cursor.write_all(&len.to_be_bytes());
        let _ =cursor.write_all(value.as_bytes());
    }
    fn write_bool(cursor:&mut Cursor<Vec<u8>>,value:bool){
        let _ = cursor.write_all(&[value as u8]);
    }
}