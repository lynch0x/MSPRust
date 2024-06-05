use std::{any::Any,collections::HashMap, io::Read};
mod amf;
mod amfstuff;
mod ticket;
mod checksum;
use ticket::*;
use crate::amfstuff::Null;
fn change_profile_picture(ticket: String,actor_id:i32,image_type:String,bytearray: Vec<u8>){
    let result =   amf::send_amf("MovieStarPlanet.WebService.Snapshots.AMFGenericSnapshotService.CreateSnapshot", vec![
        Box::new(TicketHeader{
            ticket: TicketGenerator::generate_header(ticket)
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
let result =amf::send_amf("MovieStarPlanet.WebService.User.AMFUserServiceWeb.Login", vec![
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
    panic!("Could not open file!, full error {}",x);
   }).read_to_end(&mut image_buffer).unwrap_or_else(|x|{
    panic!("Could not read file! full error: {}",x);
   });
    change_profile_picture(ticket.to_string(),*actor_id as i32,img_type,image_buffer);
    
}


//println!("Response {}",dupa.get("Response").unwrap().downcast_ref::<String>().unwrap());
//change_profile_picture(bytearray);   
   
    
    
    

    
       
}
