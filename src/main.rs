mod msp;
mod amfnew;
use std::{f64::INFINITY, thread, time::Duration};

use amfnew::AMFValue;
use msp::*;


fn get_user_input(message: &str) -> String {
    println!("{}", message);
    let mut input_buffer = String::new();
    std::io::stdin().read_line(&mut input_buffer).expect("incorrect string");
    input_buffer.trim().to_string()
}

fn login_do(login:String,password:String){
    let result = send_amf("MovieStarPlanet.WebService.User.AMFUserServiceWeb.Login", vec![
        AMFValue::STRING(login),
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
                    if let Some(status) = object.items.get("status".into()){
                        if let AMFValue::STRING(status)=status{
                            println!("Status: {}",status);
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
    login_do(login, password);
    thread::sleep(Duration::from_millis(INFINITY as u64));
}