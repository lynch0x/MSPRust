
use sha1::{Sha1, Digest};
use std::{any::Any};
use crate::ticket::TicketHeader;
use crate::amfserializer::Null;
pub struct ChecksumCalculator;

impl ChecksumCalculator{
    pub fn create_checksum(data:&Vec<Box<dyn Any>>)->String{
        let checksumable = Self::from_array(data)+"2zKzokBI4^26#oiP"+&Self::get_ticket_value(data);
        println!("Checksum string {}",checksumable);
        let mut hasher = Sha1::new();
        hasher.update(checksumable);
        let hash = hex::encode(hasher.finalize());
        println!("Checksum hash {}",hash);
        hash
    }
    pub fn get_ticket_value(data:&Vec<Box<dyn Any>>)->String{
        for item in data{
            if item.is::<TicketHeader>(){
                let ticketheader = item.downcast_ref::<TicketHeader>().unwrap();
                let podzial:Vec<&str>= ticketheader.ticket.split(',').collect();
                let koncowka = podzial.last().unwrap();
                return String::from(podzial[0])+&koncowka[koncowka.len()-5..];
            }
        }
        return  String::from("XSV7%!5!AX2L8@vn");
    }
    pub fn from_array(data:&Vec<Box<dyn Any>>)->String{
        let mut result = String::new();
        for value in data{
           
            if  value.is::<TicketHeader>(){
                continue;
            }
            if value.is::<i32>(){
                let num = *value.downcast_ref::<i32>().unwrap();
                result += &num.to_string();
            }
            if value.is::<String>(){
                result += &value.downcast_ref::<String>().unwrap().to_string();
            }
            if value.is::<bool>(){
                result += if *value.downcast_ref::<bool>().unwrap() {"True"} else {"False"}
            }
            if value.is::<Vec<u8>>(){
                result += &Self::from_byte_array(value.downcast_ref::<Vec<u8>>().unwrap());
            }
        }
        result
    }
    pub fn from_byte_array(data: &Vec<u8>)->String{
        if data.len() <= 20{
            return hex::encode(data);
        }
        let mut ar:[u8;20] = [0;20];
        let mut i= 0;
        loop {
            if i<20 {
            ar[i] = data[data.len()/20*i];
            }else{
                break;
            }
            i+=1;
        }
        return hex::encode(ar);
    }
}