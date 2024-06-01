use std::any::Any;
use amfserializer::Null;
use std::fs::File;
use std::io::{self, Read};
use reqwest::{blocking::Client, header::{HeaderMap, HeaderValue}, StatusCode};
mod amfserializer;
mod ticket;
mod checksum;
use crate::checksum::ChecksumCalculator;
use crate::ticket::TicketHeader;
use crate::ticket::TicketGenerator;
use crate::amfserializer::AMFHeader;
use crate::amfserializer::AMFMessage;
use crate::amfserializer::AMFBody;
use crate::amfserializer::AMFSerializer;
fn send_amf(method: &str,body:Vec<Box<dyn Any>>){
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
   // println!("Bytes: {:?}",stream);
    let client = Client::new();
    let mut httpheaders = HeaderMap::new();
    httpheaders.insert("Content-Type", HeaderValue::from_static("application/x-amf"));
    httpheaders.insert("Referer", HeaderValue::from_static("app:/cache/t1.bin/[[DYNAMIC]]/2"));
    httpheaders.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows; U; en) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0"));
    let result = client.post(format!("https://ws-pl.mspapis.com/Gateway.aspx?method={}",method))
        .body(stream).headers(httpheaders).send().unwrap();
    if result.status() != StatusCode::OK{
        println!("Request failed!");
    }
}
fn main(){
  
  let mut bytearray:Vec<u8> = Vec::new();
File::open("test.jpg").unwrap_or_else(|error|{
panic!("Could not read file, reason {}",error);
}).read_to_end(&mut bytearray).unwrap_or_else(|error|{
    panic!("Could not read file, reason {}",error);
});

    
    send_amf("MovieStarPlanet.WebService.Snapshots.AMFGenericSnapshotService.CreateSnapshotSmallAndBig", vec![
        Box::new(TicketHeader{
            ticket: TicketGenerator::generate_header("PL,88228438,1F2136B8-A99A-4D88-83F0-86DB6405C52B,2024-06-03T02:30:35,U5mWj/ASwGXywptpUnLSH1O0B4xQDpUZ64hdVZk3odA=,"),
            any_attribute:Null
        }),
        
        Box::new(88228438),
        Box::new(String::from("moviestar")),
        Box::new(String::from("fullSizeMovieStar")),
        Box::new(bytearray),
        Box::new(Null),
        Box::new(String::from("jpg"))]);
   
    
    
    

    
       
}
