
mod msp;
mod amfnew;
use std::{collections::BTreeMap, f64::INFINITY, thread, time::Duration};
use amfnew::{AMFValue, ASObject};

fn get_user_input(message: &str) -> String {
    println!("{}", message);
    let mut input_buffer = String::new();
    std::io::stdin().read_line(&mut input_buffer).expect("incorrect string");
    input_buffer.trim().to_string()
}
fn actorid_from_ticket(ticket: &String) -> f64 {
    let parts: Vec<&str> = ticket.split(',').collect();
    parts[1].trim().parse::<f64>().unwrap()
}

fn login_do(login:String,password:String){
    let result = msp::client::send_amf("MovieStarPlanet.WebService.User.AMFUserServiceWeb.Login", vec![
        AMFValue::STRING(login.clone()),
        AMFValue::STRING(password),
        AMFValue::NULL,
        AMFValue::NULL,
        AMFValue::NULL,
        AMFValue::STRING("MSP1-Standalone:XXXXXX".into())
    ]);
    if let Some(AMFValue::ASObject(aso)) = result {
        aso.items
            .get("loginStatus".into())
            .and_then(|login_status| {
                if let AMFValue::ASObject(object) = login_status {
                    object.items.get("ticket".into()).and_then(|ticket| {
                        if let AMFValue::STRING(ticket) = ticket {
                            Some(ticket) // Return the ticket value
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            })
            .map(|ticket| {
                let status = get_user_input("Status Text: ");
                let mut its = BTreeMap::new();
                its.insert("TextLine".into(), AMFValue::STRING(status));
                its.insert("FigureAnimation".into(), AMFValue::STRING("Boy Pose".into()));
                its.insert("ActorId".into(), AMFValue::INT(actorid_from_ticket(ticket)));
    
                msp::client::send_amf(
                    "MovieStarPlanet.WebService.ActorService.AMFActorServiceForWeb.SetMoodWithModerationCall",
                    vec![
                        msp::ticketgenerator::generate_header(ticket.clone()),
                        AMFValue::ASObject(ASObject {
                            items: its,
                            name: None,
                        }),
                        AMFValue::STRING(login),
                        AMFValue::INT(13369446.0),
                        AMFValue::BOOL(false),
                    ],
                );
                println!("Status ought to be changed!");
            })
            .unwrap_or_else(|| println!("Invalid username or password!"));
    }
    
}
fn main() {
    let login = get_user_input("Login: ");
    let password = get_user_input("Password: ");
   
    login_do(login, password);
    println!("Program execution finished.");
    thread::sleep(Duration::from_millis(INFINITY as u64));
}