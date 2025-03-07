use sha1::Sha1;

use crate::amfnew::{AMFValue, ASObject};

pub fn create_checksum(data: &Vec<AMFValue>) -> String {
    let checksumable: String = from_array(data) + "2zKzokBI4^26#oiP" + &get_ticket_value(data);
    let mut hasher = Sha1::new();
    hasher.update(checksumable.as_bytes());
    return hasher.hexdigest();
}

pub fn get_ticket_value(data: &Vec<AMFValue>) -> String {
    for item in data {
        match item{
            AMFValue::ASObject(value)=>{
                if let AMFValue::STRING(ticket) = value.items.get_key_value("Ticket").unwrap().1{
            let podzial: Vec<&str> = ticket.split(',').collect();
            if let Some(koncowka) = podzial.last() {
                return format!("{}{}", podzial[0], &koncowka[koncowka.len() - 5..]);
            }}
        }
            _=>continue
        }
    }
    "XSV7%!5!AX2L8@vn".to_string()
}
fn from_object_inner(data:&AMFValue)->String{
match data{
    AMFValue::BOOL(value)=>{
        if *value{"True".into()}else{"False".into()}
    },
    AMFValue::BYTEARRAY(value)=>from_byte_array(&value),
    AMFValue::ASObject(value)=>from_aso(&value),
    AMFValue::INT(value)=>value.to_string(),
    AMFValue::STRING(value)=>value.clone(),
    AMFValue::NULL=>String::new(),
    AMFValue::ARRAY(value)=>from_array(&value)
}
}
fn from_aso(data:&ASObject)->String{
if data.items.contains_key("Ticket"){
    return String::new();
}
let mut str = String::new();
for item in &data.items{
str += &from_object_inner(item.1);
}
str
}
pub fn from_array(data: &Vec<AMFValue>) -> String {
    data.iter()
        .filter_map(|value| {
            Some(from_object_inner(value))
        })
        .collect()
}

pub fn from_byte_array(data: &Vec<u8>) -> String {
  
    if data.len() <= 20 {
        return hex::encode(data);
    } else {
        let ar: Vec<u8> = (0..20).map(|i| data[data.len() / 20 * i]).collect();
        return hex::encode(&ar);
    }

}