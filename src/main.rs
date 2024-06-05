use std::{any::Any, io::{stdout, Read, Write}, iter::RepeatWith, net::{Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs}, thread::sleep, time::Duration};
use amfserializer::Null;
use std::collections::HashMap;
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
    let amfstream: Vec<u8> = AMFSerializer::serialize_message(&message);
let addr:SocketAddr = "ws-pl.mspapis.com:80".to_socket_addrs().unwrap().next().unwrap();
    let mut tcpclient = TcpStream::connect_timeout(&addr,Duration::from_secs(3)).expect("There was an error while connecting to host");
    let mut final_vec:Vec<u8> = Vec::new();
    final_vec.write(format!("POST /Gateway.aspx?method={} HTTP/1.1\r\n",method).as_bytes()).unwrap();
    final_vec.write("content-type: application/x-amf\r\n".as_bytes()).unwrap();
    final_vec.write("referer: app:/cache/t1.bin/[[DYNAMIC]]/2\r\n".as_bytes()).unwrap();
    final_vec.write("user-agent: Mozilla/5.0 (Windows; U; en) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0\r\n".as_bytes()).unwrap();
    final_vec.write(format!("content-length: {}\r\n",amfstream.len()).as_bytes()).unwrap();
    final_vec.write("connection: close\r\n".as_bytes()).unwrap();
    final_vec.write("host: ws-pl.mspapis.com\r\n\r\n".as_bytes()).unwrap();
    final_vec.write(&amfstream).unwrap();
    tcpclient.write_all(&final_vec).expect("Could not write body");
    let mut response = Vec::new();

tcpclient.read_to_end(&mut response).expect("Could not read stream!");

    let bodybytes =response.split(|x| *x==u8::MAX).last().unwrap();
    return amfdeserializer::AMFDeserializer::deserialize(bodybytes);

}

fn change_profile_picture(ticket: String,actor_id:i32,image_type:String,bytearray: Vec<u8>){
    let result = send_amf("MovieStarPlanet.WebService.Snapshots.AMFGenericSnapshotService.CreateSnapshot", vec![
        Box::new(TicketHeader{
            ticket: TicketGenerator::generate_header(ticket),
            any_attribute:Null
        }),
        
        Box::new(actor_id ),
        Box::new(image_type),
        Box::new(bytearray),
        Box::new("jpg")]);
    println!("Profile picture changed: {:?}",result.downcast::<bool>().unwrap());
}
fn get_user_input(message: &str)->String{
    println!("{}",message);
    let mut input_buffer:String = String::new();
    std::io::stdin().read_line(&mut input_buffer).expect("incorrect string");
    let mut return_buffer:String = String::new();
    for char in input_buffer.chars(){
        if char != 10  as char && char != 13 as char{
        return_buffer.push(char);
        }
    }
    return return_buffer;
}
fn main(){
    println!("SERVER: forced to be PL");
    let a1 = get_user_input("Username:");
    let a2 = get_user_input("Password:");
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
    let img_type = get_user_input("Image type (e.g room/moviestar/fullSizeMoviestar):");
    println!("Reading image.jpg....");
    let mut image_buffer:Vec<u8> = Vec::new();
    
   std::fs::File::open("image.jpg").unwrap_or_else(|x|{
    panic!("Could not open file!");
   }).read_to_end(&mut image_buffer).unwrap_or_else(|x|{
    panic!("Could not read file!");
   });
    change_profile_picture(ticket.to_string(),*actor_id as i32,img_type,image_buffer);
    
}


//println!("Response {}",dupa.get("Response").unwrap().downcast_ref::<String>().unwrap());
//change_profile_picture(bytearray);   
   
    
    
    

    
       
}
