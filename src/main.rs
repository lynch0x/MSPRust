mod msp;
mod amfnew;
use std::{collections::BTreeMap, thread, thread::sleep, time::Duration};
use std::f64::INFINITY;
use amfnew::{AMFValue, ASObject, AMFError};

fn get_user_input(message: &str) -> String {
    println!("{}", message);
    let mut input_buffer = String::new();
    std::io::stdin().read_line(&mut input_buffer).expect("incorrect string");
    input_buffer.trim().to_string()
}

fn actorid_from_ticket(ticket: &str) -> f64 {
    ticket.split(',')
        .nth(1)
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn login_do(login: String, password: String) -> Result<(), AMFError> {
    let result = msp::client::send_amf(
        "MovieStarPlanet.WebService.User.AMFUserServiceWeb.Login",
        AMFValue::ARRAY(vec![
            AMFValue::STRING(login.clone()),
            AMFValue::STRING(password),
            AMFValue::NULL,
            AMFValue::NULL,
            AMFValue::NULL,
            AMFValue::STRING("MSP1-Standalone:XXXXXX".into()),
        ])
    )?;


    let login_status = result.as_object().items.get("loginStatus")
        .ok_or(AMFError::ValueNotFound)?;

    let status = login_status.as_object().items.get("status")
        .ok_or(AMFError::ValueNotFound)?
        .as_string();

    if status != "Success" {
        println!("LoginStatus: {}",status);
        return Ok(());
    }

    let ticket = login_status.as_object().items.get("ticket")
        .ok_or(AMFError::ValueNotFound)?
        .as_string();

    let status_text = get_user_input("Status Text: ");

    let mut items = BTreeMap::new();
    items.insert("TextLine".into(), AMFValue::STRING(status_text));
    items.insert("FigureAnimation".into(), AMFValue::STRING("Boy Pose".into()));
    items.insert("ActorId".into(), AMFValue::INT(actorid_from_ticket(ticket)));

    let response = msp::client::send_amf(
        "MovieStarPlanet.WebService.ActorService.AMFActorServiceForWeb.SetMoodWithModerationCall",
        AMFValue::ARRAY(vec![
            msp::ticketgenerator::generate_header(ticket),
            AMFValue::ASObject(ASObject {
                items,
                name: None,
            }),
            AMFValue::STRING(login),
            AMFValue::INT(13369446.0),
            AMFValue::BOOL(false),
        ])
    )?;

    println!("Mood changed successfully!");
    Ok(())
}

fn main() {
    let login = get_user_input("Login: ");
    let password = get_user_input("Password: ");


    let result: &'static str = match login_do(login, password) {
        Err(e) => e.as_static_str(),
        Ok(()) => "Program finished execution"
    };
    println!("Result: {}", result);
    sleep(Duration::from_millis(u64::MAX));
}