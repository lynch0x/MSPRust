
use std::{any::Any, collections::HashMap,  time::Duration};

use crate::amfserializer::Null;
pub struct AMFDeserializer;
impl AMFDeserializer{

    pub fn deserialize(bytes: &[u8])->Box<dyn Any>{
        let mut pos:u32 = 0;
        return Self::read_data(bytes, &mut pos);
    }
    pub fn read_data(bytes:&[u8],pos:&mut u32)-> Box<dyn Any>{

        let marker = Self::read_bytes(bytes, pos,1)[0];
        return Self::read_data_from_marker(bytes, pos, marker);
    }
    pub fn read_data_from_marker(bytes:&[u8],pos:&mut u32,marker:u8)-> Box<dyn Any>{
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
    pub fn read_object(bytes:&[u8],pos:&mut u32)->HashMap<String,Box<dyn Any>>{
        let obname_useless = Self::read_string(bytes, pos);
        return Self::read_aso(bytes, pos);
    }
    pub fn read_array(bytes:&[u8],pos:&mut u32)->Vec<Box<dyn Any>>{
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
    pub fn read_associative_array(bytes:&[u8],pos:&mut u32)->HashMap<String,Box<dyn Any>>{
        let length_useless = Self::read_int32(bytes, pos);
        return Self::read_aso(bytes, pos);
    }
    pub fn read_int32(bytes:&[u8],pos:&mut u32)->i32{
        let bytes = Self::read_bytes(bytes, pos, 4);

        return i32::from_be_bytes([bytes[0],bytes[1],bytes[2],bytes[3]]);
    }
    pub fn read_aso(bytes:&[u8],pos:&mut u32)->HashMap<String,Box<dyn Any>>{
        let mut map: HashMap<String, Box<dyn Any>> = HashMap::new();
        loop{
            let name = Self::read_string(bytes, pos);
            let marker = Self::read_bytes(bytes, pos,1)[0];
            if marker == 9{
                break;
            }
            map.insert(name, Self::read_data_from_marker(bytes, pos, marker));
        }
        let for_debug = map;
        return for_debug;
    }
    pub fn read_uint16(bytes:&[u8],pos:&mut u32)->u16{
        let bytes = Self::read_bytes(bytes, pos, 2);
        return u16::from_be_bytes([bytes[0],bytes[1]]);
    }
    pub fn read_string(bytes:&[u8],pos:&mut u32)->String{
        let len = Self::read_uint16(bytes, pos);
        let bytes2 = Self::read_bytes(bytes, pos, len);
        return String::from_utf8(bytes2).unwrap();
    }
    pub fn read_boolean(bytes:&[u8],pos:&mut u32)->bool{
        if Self::read_bytes(bytes, pos, 1)[0] == 1 {return true} else {return false};
    }
pub fn read_double(bytes:&[u8],pos:&mut u32)->f64{
   
    let array = Self::read_bytes(bytes, pos, 8);
    let mut array2 = [0u8; 8];
    let mut num2 = 0;
    for num in (0..8).rev() {
        array2[num2] = array[num];
        num2 += 1;
    }

    // Convert the byte array to a f64
    f64::from_bits(u64::from_le_bytes(array2))
}
pub fn read_bytes(bytes:&[u8],pos:&mut u32, len: u16)->Vec<u8>{
//    *pos += 1;
    let mut buffer:Vec<u8> = Vec::new();
    let mut i:usize = 0;
    loop{
        if i>=len as usize {break;}
        buffer.push(bytes[*pos as usize]);
        *pos += 1;
        i+=1;
    }

    return buffer;
}
}