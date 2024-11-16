mod msp;
mod amfnew;
use std::{collections::BTreeMap, f64::INFINITY, thread, time::Duration};

use amfnew::{AMFValue, ASObject};
use msp::*;


fn get_user_input(message: &str) -> String {
    println!("{}", message);
    let mut input_buffer = String::new();
    std::io::stdin().read_line(&mut input_buffer).expect("incorrect string");
    input_buffer.trim().to_string()
}

fn login_do(login:String,password:String,status:String){
    let result = send_amf("MovieStarPlanet.WebService.User.AMFUserServiceWeb.Login", vec![
        AMFValue::STRING(login.clone()),
        AMFValue::STRING(password),
        AMFValue::NULL,
        AMFValue::NULL,
        AMFValue::NULL,
        AMFValue::STRING("MSP1-Standalone:XXXXXX".into())
    ]);
    if let Some(value) = result{
        if let AMFValue::ASObject(aso) = value{
            if let Some(object) = aso.items.get("loginStatus".into()){
                if let AMFValue::ASObject(object) = object{
                    if let Some(ticket) = object.items.get("ticket".into()){
                        if let AMFValue::STRING(ticket)=ticket{
                            let mut its = BTreeMap::new();
                            its.insert("TextLine".into(), AMFValue::STRING(status));
                            its.insert("FigureAnimation".into(), AMFValue::STRING("Boy Pose".into()));
                           its.insert("ActorId".into(), AMFValue::INT(88536331.0));
                            send_amf("MovieStarPlanet.WebService.ActorService.AMFActorServiceForWeb.SetMoodWithModerationCall", vec![
                                TicketGenerator::generate_header(ticket.clone()),
                                AMFValue::ASObject(ASObject{
                                    items:its,
                                    name:None
                                }),
                                AMFValue::STRING(login),
                                AMFValue::INT(0.0),
                                AMFValue::BOOL(false)
                            ]);
                            println!("Ticket: {}",ticket);
                            println!("Status ought to be changed!");
                        }else{
                            println!("Invalid username or password!");
                        }
                       
                    }
                }
            }
        }
    }
}
fn main() {
    let login = get_user_input("Login: ");
    let password = get_user_input("Password: ");
    let status = get_user_input("Status Text: ");
    login_do(login, password,status);
    println!("Program execution finished.");
    thread::sleep(Duration::from_millis(INFINITY as u64));
}