use std::any::Any;

use crate::ticket::TicketHeader;

pub struct  AMFHeader{
    pub name: String,
    pub must_understand: bool,
    pub body: Box<dyn Any>
}
pub struct AMFBody{
    pub target:String,
    pub response:String,
    pub body:Box<dyn Any>
}
pub struct AMFMessage{
    pub headers: Vec<AMFHeader>,
    pub body: AMFBody,
    pub version: u16
}

pub struct AMFSerializer;

impl AMFSerializer{
    pub fn serialize_message(message: &AMFMessage)->Vec<u8>{
        let mut source: Vec<u8> = Vec::new();
        let stream = &mut source;
        Self::write_uint16(stream, 0);
        Self::write_uint16(stream, message.headers.len() as u16);
        for header in &message.headers{
            Self::write_utf8(stream, &header.name);
            Self::write_boolean(stream, header.must_understand);
            Self::write_int32(stream, -1);
            Self::write_data(stream, &header.body);
        }
       Self::write_uint16(stream, 1);
       Self::write_utf8(stream, &message.body.target);
       Self::write_utf8(stream, &message.body.response);
       Self::write_int32(stream, -1);
       Self::write_data(stream, &message.body.body);
       return source;
    }
    pub fn write_end_markup(stream: &mut Vec<u8>){
        stream.push(0);
        stream.push(0);
        stream.push(9);
    }
    pub fn write_data(stream: &mut Vec<u8>, value: &Box<dyn Any>){
        if value.is::<Vec<Box<dyn Any>>>(){
            Self::write_array(stream, value.downcast_ref::<Vec<Box<dyn Any>>>().unwrap())
        }
        else if  value.is::<Null>(){
            Self::write_null(stream);
        }
        else if value.is::<String>(){
            stream.push(2);
            Self::write_utf8(stream, value.downcast_ref::<String>().unwrap());
        }
        else if value.is::<&str>(){
            stream.push(2);
            Self::write_utf8_str(stream, *value.downcast_ref::<&str>().unwrap());
        }
        else if value.is::<bool>(){
            stream.push(1);
            Self::write_boolean(stream, *value.downcast_ref::<bool>().unwrap());
        }
        else if value.is::<i32>(){
            stream.push(0);
            Self::write_double(stream, *value.downcast_ref::<i32>().unwrap() as f64);
        }
        else if value.is::<TicketHeader>(){
            stream.push(3);
            Self::write_utf8(stream, &String::from("Ticket"));
            stream.push(2);
            Self::write_utf8(stream, &value.downcast_ref::<TicketHeader>().unwrap().ticket);
            Self::write_utf8(stream, &String::from("anyAttribute"));
            Self::write_null(stream);
            Self::write_end_markup(stream);
        }else if value.is::<Vec<u8>>(){
            stream.push(17); //change amf encoding to AMF3
            stream.push(12); //byte array marker
            let array = value.downcast_ref::<Vec<u8>>().unwrap();
            let mut len = array.len() as u32;
            len <<=1;
            len |=1;
            Self::write_amf3_uint32_data(stream, &mut len);
            for i in array{
                stream.push(*i);
            }
        }
    }
    pub fn write_amf3_uint32_data(stream:&mut Vec<u8>, value:&mut u32){
        *value &=536870911;
        if *value <128 { stream.push(*value as u8);return;}
        if *value <16384{
             stream.push(((*value >> 7 &127) | 128) as u8);
             stream.push((*value &127) as u8);
             return;
            }
        if *value <2097152{
            stream.push(((*value>>14&127)|128) as u8);
            stream.push(((*value>>7&127)|128) as u8);
            stream.push((*value &127) as u8);
            return;
        }
        //TODO implement larger int
    }
    pub fn write_array(stream:&mut Vec<u8>,value:&Vec<Box<dyn Any>>){
        stream.push(10);
        Self::write_int32(stream, value.len() as i32);
        for item in value{
            Self::write_data(stream, item);
        }
    }
    pub fn write_double(stream:&mut Vec<u8>,value:f64){
        Self::write_bytes(stream, &value.to_be_bytes());
    }
    pub fn write_null(stream:&mut Vec<u8>){
        stream.push(5);
    }
    pub fn write_int32(stream:&mut Vec<u8>,value:i32){
        Self::write_bytes(stream, &value.to_be_bytes());
    }
    pub fn write_boolean(stream: &mut Vec<u8>, boolean: bool){
        stream.push(boolean as u8);
    }
    pub fn write_uint16(stream: &mut Vec<u8>,value: u16){
        let bytes:[u8;2] = value.to_be_bytes();
        Self::write_bytes(stream,&bytes);
    }
  pub fn write_utf8(stream: &mut Vec<u8>, value:&String){
    let bytes = value.as_bytes();
    Self::write_uint16(stream, bytes.len() as u16);
    Self::write_bytes(stream, bytes);
  }
  pub fn write_utf8_str(stream: &mut Vec<u8>, value:&str){
    let bytes = value.as_bytes();
 
    Self::write_uint16(stream, bytes.len() as u16);
    Self::write_bytes(stream, bytes);
  }
    pub fn write_bytes(stream: &mut Vec<u8>,bytes: &[u8]){ for item in bytes { stream.push(*item);} }
} 
pub struct Null;
