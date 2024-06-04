use std::{any::Any, io::{stdout, Read, Write}, thread::sleep, time::Duration};
use amfserializer::Null;
use std::collections::HashMap;
use reqwest::{blocking::Client, header::{HeaderMap, HeaderValue}, StatusCode};
mod amfserializer;
mod ticket;
mod checksum;
mod amfdeserializer;
use crate::checksum::ChecksumCalculator;
use crate::ticket::TicketHeader;
use crate::ticket::TicketGenerator;
use crate::amfserializer::AMFHeader;
use crate::amfserializer::AMFMessage;
use crate::amfserializer::AMFBody;
use crate::amfserializer::AMFSerializer;
fn send_amf(method: &str,body:Vec<Box<dyn Any>>)->Box<dyn Any>{
    let message:AMFMessage = AMFMessage{
        headers: vec![
            AMFHeader{
            name: String::from("needClassName"),
            must_understand: false,
            body: Box::new(false)
            }
        ,
        AMFHeader{
            name: String::from("id"),
            must_understand: false,
            body: Box::new(ChecksumCalculator::create_checksum(&body))
            }],
        version: 0,
        body: AMFBody{
            target:String::from(method),
            response:String::from("/1"),
            body:Box::new(body)
        }
    };
    let stream: Vec<u8> = AMFSerializer::serialize_message(&message);
 //   println!("Bytes: {:?}",stream);
    let client = Client::new();
    let mut httpheaders = HeaderMap::new();
    httpheaders.insert("Content-Type", HeaderValue::from_static("application/x-amf"));
    httpheaders.insert("Referer", HeaderValue::from_static("app:/cache/t1.bin/[[DYNAMIC]]/2"));
    httpheaders.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows; U; en) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0"));
    let result = client.post(format!("https://ws-pl.mspapis.com/Gateway.aspx?method={}",method))
        .body(stream).headers(httpheaders).send().unwrap();
    if result.status() == StatusCode::OK{
        let bytes =result.bytes().unwrap();
        let bodybytes =bytes.split(|x| *x==u8::MAX).last().unwrap();
       // println!("Bytes: {:?}",bodybytes);
        return amfdeserializer::AMFDeserializer::deserialize(bodybytes);
       
      //  Err("Request failed");
    }
    return Box::new(Null);
     //Ok(amfdeserializer::AMFDeserializer::deserialize(result.bytes().unwrap().split(|x| *x==u8::MAX).last().unwrap()));
}

fn change_profile_picture(ticket: String,actor_id:i32,bytearray: Vec<u8>){
    let result = send_amf("MovieStarPlanet.WebService.Snapshots.AMFGenericSnapshotService.CreateSnapshot", vec![
        Box::new(TicketHeader{
            ticket: TicketGenerator::generate_header(ticket),
            any_attribute:Null
        }),
        
        Box::new(actor_id ),
        Box::new("moviestar"),
        Box::new(bytearray),
        Box::new("jpg")]);
    println!("Profile picture changed: {:?}",result.downcast::<bool>().unwrap());
}

fn main(){
    println!("SERVER: forced to be PL");
    let mut username_buffer:String = String::new();
    let mut password_buffer:String = String::new();
    println!("Username: ");
    std::io::stdin().read_line(&mut username_buffer).expect("incorrect string");
    println!("Password: ");
  
    std::io::stdin().read_line(&mut password_buffer).expect("incorrect string");
    let mut a1:String = String::new();
    let mut a2:String = String::new();
    for char in username_buffer.chars(){
        if char != 10  as char && char != 13 as char{
        a1.push(char);
        }
    }
    for char in password_buffer.chars(){
        if char != 10  as char && char != 13 as char{
        a2.push(char);
        }
    }
let result =send_amf("MovieStarPlanet.WebService.User.AMFUserServiceWeb.Login", vec![
    Box::new(a1),
    Box::new(a2),
    Box::new(Null),
    Box::new(Null),
    Box::new(Null),
    Box::new("MSP1-Standalone:XXXXXX")
  ]);
 let dupa = result.downcast_ref::<HashMap<String,Box<dyn Any>>>().unwrap();
  let login_status = dupa.get("loginStatus").unwrap().downcast_ref::<HashMap<String,Box<dyn Any>>>().unwrap();
  let status =login_status.get("status").unwrap().downcast_ref::<String>().unwrap();
println!("Status: {}",status);
if status == "Success"{
   let ticket=login_status.get("ticket").unwrap().downcast_ref::<String>().unwrap();
   let actor_id=login_status.get("actor").unwrap().downcast_ref::<HashMap<String,Box<dyn Any>>>().unwrap().get("ActorId").unwrap().downcast_ref::<f64>().unwrap();
    println!("Your ticket {}",ticket);

    let mut image_buffer:Vec<u8> = Vec::new();
    
   std::fs::File::open("image.jpg").unwrap().read_to_end(&mut image_buffer).unwrap();
    change_profile_picture(ticket.to_string(),*actor_id as i32,image_buffer);
    
}
sleep(Duration::new(6,0));

//println!("Response {}",dupa.get("Response").unwrap().downcast_ref::<String>().unwrap());
//change_profile_picture(bytearray);   
   
    
    
    

    
       
}
