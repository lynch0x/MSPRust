use std::any::Any;
use std::collections::HashMap;
use crate::ticket::TicketHeader;
use std::time::Duration;
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

pub struct AMFDeserializer;
impl AMFDeserializer{

    pub fn deserialize(bytes: &[u8])->Box<dyn Any>{
        let mut pos:usize = 0;
        return Self::read_data(bytes, &mut pos);
    }
    pub fn read_data(bytes:&[u8],pos:&mut usize)-> Box<dyn Any>{

        let marker = Self::read_bytes::<1>(bytes, pos)[0];
        return Self::read_data_from_marker(bytes, pos, marker);
    }
    pub fn read_data_from_marker(bytes:&[u8],pos:&mut usize,marker:u8)-> Box<dyn Any>{
        if marker == 0{
            return Box::new(Self::read_double(bytes, pos));
        }
        if marker == 1{
            return Box::new(Self::read_boolean(bytes,pos));
        }
        if marker == 2{
            return Box::new(Self::read_string(bytes, pos));
        }
        if marker ==3{
            return Box::new(Self::read_aso(bytes, pos));
        }
        if marker == 5{
            return Box::new(Null);
        }
        if marker == 8{
            return Box::new(Self::read_associative_array(bytes, pos));
        }
        if marker == 10{
            let ar=Self::read_array(bytes, pos);
            return Box::new(ar);
        }
        if marker == 11{
        //    println!("Skipping deserializing datetime. Returning as empty Duration.");
            *pos+=10;
            return Box::new(Duration::new(0, 0));
        }
        if marker == 16{
            return Box::new(Self::read_object(bytes, pos));
        }
       panic!("unknown type at {}",*pos);
    }
    pub fn read_object(bytes:&[u8],pos:&mut usize)->HashMap<String,Box<dyn Any>>{
         Self::read_string(bytes, pos);
        return Self::read_aso(bytes, pos);
    }
    pub fn read_array(bytes:&[u8],pos:&mut usize)->Vec<Box<dyn Any>>{
        let length = Self::read_int32(bytes, pos);
        let mut i:i32 = 0;
        let mut vec:Vec<Box<dyn Any>> = Vec::new();
        loop{
            if i>=length{break;}
            vec.push(Self::read_data(bytes, pos));
            i+=1;
        }
        return vec;
    }
    pub fn read_associative_array(bytes:&[u8],pos:&mut usize)->HashMap<String,Box<dyn Any>>{
       *pos+=4;
        return Self::read_aso(bytes, pos);
    }
    pub fn read_int32(bytes:&[u8],pos:&mut usize)->i32{
        let bytes = Self::read_bytes::<4>(bytes, pos);

        return i32::from_be_bytes(bytes);
    }
    pub fn read_aso(bytes:&[u8],pos:&mut usize)->HashMap<String,Box<dyn Any>>{
        let mut map: HashMap<String, Box<dyn Any>> = HashMap::new();
        loop{
            let name = Self::read_string(bytes, pos);
            let marker = Self::read_bytes::<1>(bytes, pos)[0];
            if marker == 9{
                break;
            }
            map.insert(name, Self::read_data_from_marker(bytes, pos, marker));
        }
        let for_debug = map;
        return for_debug;
    }
    pub fn read_uint16(bytes:&[u8],pos:&mut usize)->u16{
        let bytes = Self::read_bytes::<2>(bytes, pos);
        return u16::from_be_bytes(bytes);
    }
    pub fn read_string(bytes:&[u8],pos:&mut usize)->String{
        let len = Self::read_uint16(bytes, pos);
        let bytes2 = Self::read_bytes_slow(bytes, pos,len as usize);
        return String::from_utf8(bytes2).unwrap();
    }
    pub fn read_boolean(bytes:&[u8],pos:&mut usize)->bool{
       return  Self::read_bytes::<1>(bytes, pos)[0] != 0;
    }
pub fn read_double(bytes:&[u8],pos:&mut usize)->f64{
   
    let array = Self::read_bytes::<8>(bytes, pos);
    return f64::from_be_bytes(array);
}
pub fn read_bytes<const N:usize>(bytes:&[u8],pos:&mut usize)->[u8;N]{
//    *pos += 1;
    let mut buffer:[u8;N] = [0;N];
    let mut i:usize = 0;
    loop{
        if i>=N {break;}
        buffer[i] = bytes[*pos];
        *pos += 1;
        i+=1;
    }

    return buffer;
}
pub fn read_bytes_slow(bytes:&[u8],pos:&mut usize,len:usize)->Vec<u8>{
    //    *pos += 1;
        let mut buffer:Vec<u8> = Vec::new();
        let mut i:usize = 0;
        loop{
            if i>=len {break;}
            buffer.push(bytes[*pos]);
            *pos += 1;
            i+=1;
        }
    
        return buffer;
    }
}