use std::{any::Any, collections::HashMap, fs::File, io::Read};
mod amf;
use crate::amf::{send_amf,TicketGenerator,TicketHeader,Null};

fn change_profile_picture(ticket: String, actor_id: i32, image_type: String, bytearray: Vec<u8>) {
    let result = send_amf(
        "MovieStarPlanet.WebService.Snapshots.AMFGenericSnapshotService.CreateSnapshot",
        vec![
            Box::new(TicketHeader {
                ticket: TicketGenerator::generate_header(ticket),
            }),
            Box::new(actor_id),
            Box::new(image_type),
            Box::new(bytearray),
            Box::new("jpg"),
        ],
    );
    println!("Profile picture changed: {:?}", result.downcast::<bool>().unwrap());
}

fn get_user_input(message: &str) -> String {
    println!("{}", message);
    let mut input_buffer = String::new();
    std::io::stdin().read_line(&mut input_buffer).expect("incorrect string");
    input_buffer.trim().to_string()
}

fn main() {
    println!("SERVER: forced to be PL");
    let username = get_user_input("Username:");
    let password = get_user_input("Password:");

    let result = send_amf(
        "MovieStarPlanet.WebService.User.AMFUserServiceWeb.Login",
        vec![
            Box::new(username),
            Box::new(password),
            Box::new(Null),
            Box::new(Null),
            Box::new(Null),
            Box::new("MSP1-Standalone:XXXXXX"),
        ],
    );

    let login_status = result.downcast_ref::<HashMap<String, Box<dyn Any>>>()
        .and_then(|res| res.get("loginStatus"))
        .and_then(|login_status| login_status.downcast_ref::<HashMap<String, Box<dyn Any>>>())
        .unwrap();

    let status = login_status.get("status")
        .and_then(|status| status.downcast_ref::<String>())
        .unwrap();

    println!("Status: {}", status);

    if status == "Success" {
        let ticket = login_status.get("ticket")
            .and_then(|ticket| ticket.downcast_ref::<String>())
            .unwrap_or_else(|| panic!("Ticket not found"));

        let actor_id = login_status.get("actor")
            .and_then(|actor| actor.downcast_ref::<HashMap<String, Box<dyn Any>>>())
            .and_then(|actor| actor.get("ActorId"))
            .and_then(|actor_id| actor_id.downcast_ref::<f64>())
            .map(|actor_id| *actor_id as i32)
            .unwrap_or_else(|| panic!("ActorId not found"));

        println!("Your ticket {}", ticket);
        let img_type = get_user_input("Image type (e.g room/moviestar/fullSizeMoviestar):");
        println!("Reading image.jpg....");

        let mut image_buffer = Vec::new();
        File::open("image.jpg")
            .unwrap_or_else(|err| panic!("Could not open file! Error: {}", err))
            .read_to_end(&mut image_buffer)
            .unwrap_or_else(|err| panic!("Could not read file! Error: {}", err));

        change_profile_picture(ticket.to_string(), actor_id, img_type, image_buffer);
    }
}