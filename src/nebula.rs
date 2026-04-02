use base64::Engine;
use std::collections::HashMap;
use std::fmt::format;
use crate::httpwindows::net_connection;

const USA_SERVERS: [&str; 4] = ["AU", "NZ", "CA", "US"];
pub(crate) const SUPPORTED_SERVERS:[&'static str;16] = ["AU","CA","DE","DK","ES","FR","IE","NL","NZ","NO","PL","FI","SE","TR","GB","US"];
#[inline]
pub fn server_check(server:&str)->bool{

    SUPPORTED_SERVERS.contains(&server)
}
#[derive(Clone, PartialEq,Debug)]
pub enum LoginState {
    LOGGEDIN,
    RATELIMITED,
    INVALID,
    FAIL
}

impl Default for LoginState {
    fn default() -> Self {
        LoginState::INVALID
    }
}

#[derive(Clone, Default,Debug,PartialEq)]
pub struct LoginResult {
    pub login_state: LoginState,
    pub access_token: String,
    pub refresh_token: String,
    pub profile_id: String,
    pub password: String,
    pub server: String,
    pub username: String,
}

fn extract_json_value(json_str: &str, key: &'static str) -> Option<String> {
    let key_pattern = format!("\"{}\":\"", key);

    if let Some(start) = json_str.find(&key_pattern) {
        let token_start = start + key_pattern.len();

        if let Some(end) = json_str[token_start..].find('\"') {
            return Some(json_str[token_start..token_start + end].to_string());
        }
    }

    None
}

impl LoginResult {
    #[inline]
    pub fn is_us_server(&self) -> bool {
        USA_SERVERS.contains(&self.server.as_str())
    }


}




#[inline]
fn get_url_endpoint(lr: &LoginResult) -> &'static str {
    if lr.is_us_server() { "us" } else { "eu" }
}

fn get_tokens_from_json(json_str: &str) -> Result<(String, String), &'static str> {
    let access_token =
        extract_json_value(json_str, "access_token").ok_or("Missing access_token")?;
    let refresh_token =
        extract_json_value(json_str, "refresh_token").ok_or("Missing refresh_token")?;
    Ok((access_token, refresh_token))
}

fn login_id_from_access_token(token: &str) -> String {
    let split: Vec<&str> = token.split('.').collect();
    let text = split[1];
    let mut fixed_base64 = text.replace('-', "+").replace('_', "/");
    let padding = (4 - fixed_base64.len() % 4) % 4;
    fixed_base64.push_str(&"=".repeat(padding));

    let decoded_base64_bytes = base64::engine::general_purpose::STANDARD
        .decode(fixed_base64)
        .unwrap();
    let decoded_base64_string = String::from_utf8(decoded_base64_bytes).unwrap();

    let split: Vec<&str> = decoded_base64_string.split('"').collect();
    split[19].to_string()
}

fn get_profile_id_and_username_from_token(lr: &LoginResult) -> Result<(String,String),()> {
    let login_id = login_id_from_access_token(&lr.access_token);

    let host = format!(
        "{}.mspapis.com",
        get_url_endpoint(lr)
    );
    let path = format!("/profileidentity/v1/logins/{}/profiles",login_id);
    let headers_string = format!(
        "Authorization: Bearer {}\r\n",
        lr.access_token
    );
    let resp = unsafe {
        net_connection(&host, &path,"GET",Some(headers_string.as_bytes()), None,None)
    };
    match resp{
        Ok(val)=>{
            let string = String::from_utf8_lossy(val.as_slice());
            Ok((extract_json_value(&string, "id").unwrap(),extract_json_value(&string, "name").unwrap()))
        },
        Err(_)=>Err(())
    }


}

fn login_to_nebula(lr: &mut LoginResult,proxy:Option<&str>) {
    let host = format!(
        "{}-secure.mspapis.com",
        get_url_endpoint(lr)
    );
    let path = "/loginidentity/connect/token";
    let tmp = format!("{}|{}", lr.server, lr.username);
    let body_string = format!(
        "client_id=unity.client&grant_type=password&scope={}&username={}&password={}",
        "openid%20nebula%20offline_access", // Spaces become %20 or +
        tmp,
        lr.password
    );

    let resp = unsafe {
        net_connection(&host, &path,"POST",Some("Content-Type: application/x-www-form-urlencoded\r\n".as_bytes()), Some(body_string.as_bytes()),proxy)
    };
    let response_text:String = match resp{
        Ok(val)=>{
            String::from_utf8_lossy(val.as_slice()).to_string()

        },
        Err(_)=>String::new()
    };

    if response_text.contains("many") {
        lr.login_state = LoginState::RATELIMITED;
    } else if let Ok(tokens) = get_tokens_from_json(&response_text) {
        lr.login_state = LoginState::LOGGEDIN;
        lr.access_token = tokens.0;
        lr.refresh_token = tokens.1;
    }else{
        println!("{}",response_text);
        lr.login_state = LoginState::FAIL;
    }
}

pub fn refresh_token(data: &mut LoginResult,proxy:Option<&str>) {
    let host = format!(
        "{}-secure.mspapis.com",
        get_url_endpoint(data)
    );
    let path = "/loginidentity/connect/token";


    let body_string = format!(
        "scope=openid%20nebula%20offline%5Faccess&username={}%7C{}&client%5Fid=unity%2Eclient&acr%5Fvalues=profileId%3A{}%20gameId%3A5ooi%20deviceid%3A185f5566%2Dcc85%2Dccd0%2De7a7%2Dad52ae136d8d&grant%5Ftype=password&password={}",
        data.server,
        data.username,
        data.profile_id,
        data.password
    );

    let resp = unsafe {
        net_connection(&host, &path,"POST",Some("Content-Type: application/x-www-form-urlencoded\r\n".as_bytes()), Some(body_string.as_bytes()),proxy)
    };
    let response_text:String = match resp{
        Ok(val)=>{
            String::from_utf8_lossy(val.as_slice()).to_string()

        },
        Err(_)=>String::new()
    };
    if response_text.contains("many") {
        data.login_state = LoginState::RATELIMITED;
    } else if let Ok(tokens) = get_tokens_from_json(&response_text) {
        data.access_token = tokens.0;
        data.refresh_token = tokens.1;
    }
}

pub fn login(server: String, username: String, password: String,proxy:Option<&str>,refresh:bool) -> LoginResult {
    let mut lr = LoginResult::default();
    lr.username = username;
    lr.password = password;
    lr.server = server;

    login_to_nebula(&mut lr,proxy);
if refresh {
    if lr.login_state == LoginState::LOGGEDIN {
        if let Ok(result) = get_profile_id_and_username_from_token(&lr) {
            lr.profile_id = result.0;
            lr.username = result.1;
            refresh_token(&mut lr, proxy);
        } else {
            lr.login_state = LoginState::FAIL;
        }
    }
}

    lr
}
