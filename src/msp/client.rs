use std::fs;
use crate::amfnew::*;
use crate::httpwindows::net_connection;
use super::checksumcalculator;
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
fn generate_id() -> String {
    let mut rng = rand::rng();
    let mut s = String::new();

    while s.len() < 48 {
        let num: u32 = rng.random(); // odpowiednik Math.random() * int.MAX_VALUE
        s.push_str(&format!("{:x}", num));
    }

    let trimmed = &s[..46];
    general_purpose::STANDARD.encode(trimmed)
}
pub fn send_amf(method: &str, body: AMFValue,proxy:Option<&str>) -> Result<AMFValue, AMFError> {
    let message = AMFMessage {
        headers: vec![
            AMFHeader{
                name:String::from("sessionID"),
                must_understand:false,
                body:AMFValue::STRING(String::from(generate_id()))
            },
            AMFHeader {
                name: String::from("id"),
                must_understand: false,
                body: AMFValue::STRING(checksumcalculator::create_checksum(&body))
            }
            ,
            AMFHeader {
                name: String::from("needClassName"),
                must_understand: false,
                body: AMFValue::BOOL(true),
            }

        ],
        version: 0,
        body: AMFBody {
            target: String::from(method),
            response: String::from("/1"),
            body
        },
    };

    let amf_stream = AMFSerializer::serialize_amf_message(message);
    let mut path = String::new();
    path += "/Gateway.aspx?method=";
    path += method;
    // Używamy sufiksu 'b' dla tablicy bajtów (u8), co jest bezpieczniejsze dla protokołów binarnych
    const HEADERS: &[u8] = b"x-flash-version: 32,0,0,100\r\n\
                         Content-Type: application/x-amf\r\n\
                         Accept-Encoding: gzip, deflate\r\n";
    let resp = unsafe {
        net_connection("ws-pl.moviestarplanet.app", &path,"POST",Some(HEADERS), Some(amf_stream.as_slice()),proxy)
    }.map_err(|_| AMFError::RequestFailed)?;
    fs::write("response.bin", &resp).expect("Nie udało się zapisać pliku");
    AMFDeserializer::deserialize_amf_message(resp)
}
