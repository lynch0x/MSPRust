use core::hash;
use std::{collections::HashMap, io::{Read, Write}, net::{SocketAddr, TcpStream, ToSocketAddrs}, sync::Arc, time::{Duration, SystemTime}};


use rustls::{ClientConfig, ClientConnection};
use sha1::Sha1;

use crate::amfnew::*;

fn send_request(method: &str, amfstream: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {

    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(
        webpki_roots::TLS_SERVER_ROOTS
            .0
            .iter()
            .map(|ta| {
                rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                    ta.subject,
                    ta.spki,
                    ta.name_constraints,
                )
            })
    );
    
    let config = rustls::ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(root_store)
    .with_no_client_auth();

    let server_name = "ws-pl.mspapis.com".try_into()?;
    let mut connector = ClientConnection::new(Arc::new(config), server_name)?;

    // Establishing a TCP connection with a timeout
    let addr: SocketAddr = "ws-pl.mspapis.com:443"
        .to_socket_addrs()?
        .next()
        .ok_or("Failed to resolve address")?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))?;

    stream.set_nonblocking(false)?;

    let mut tls_stream = rustls::Stream::new(&mut connector, &mut stream);

    // Prepare HTTP request
    let mut final_vec = Vec::new();
    final_vec.write_all(format!("POST /Gateway.aspx?method={} HTTP/1.0\r\n", method).as_bytes())?;
    final_vec.write_all(b"content-type: application/x-amf\r\n")?;
    final_vec.write_all(b"referer: app:/cache/t1.bin/[[DYNAMIC]]/2\r\n")?;
    final_vec.write_all(b"user-agent: Mozilla/5.0 (Windows; U; en) AdobeAIR/32.0\r\n")?;
    final_vec.write_all(format!("content-length: {}\r\n", amfstream.len()).as_bytes())?;
    final_vec.write_all(b"connection: close\r\n")?;
    final_vec.write_all(b"host: ws-pl.mspapis.com\r\n\r\n")?;
    final_vec.write_all(&amfstream)?;

    tls_stream.write_all(&final_vec)?;

    tls_stream.flush()?;

  
    let mut response = Vec::new();
    
    loop {
        let mut buf = [0; 1024];
        match tls_stream.read(&mut buf) {
            Ok(0) => break, // Connection closed
            Ok(n) => response.extend_from_slice(&buf[..n]),
            Err(ref e)=> {
                if e.kind() == std::io::ErrorKind::WouldBlock{ continue;}
                if e.kind() == std::io::ErrorKind::UnexpectedEof //to znaczy ze serwer skonczyl wysylac response, nvm dlaczego jest uznawane jako blad
                {
                    break;
                }
            }
           
    }
    }
    let http_headers_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap_or_else(|| panic!("Could not read http headers end"));

    let body = &response[http_headers_end + 4..];
    if body.is_empty() {
        panic!("Response body is null!");
    }
    Ok(body.to_vec())
}
pub fn send_amf(method: &str,body:Vec<AMFValue>)->Option<AMFValue>{
    let message = AMFMessage {
        headers: vec![
            AMFHeader {
                name: String::from("needClassName"),
                must_understand: false,
                body: AMFValue::BOOL(false),
            },
            AMFHeader{
                name:String::from("id"),
                must_understand:false,
                body: AMFValue::STRING(ChecksumCalculator::create_checksum(&body))
            }
            
        ],
        version: 0,
        body: AMFBody {
            target: String::from(method),
            response: String::from("/1"),
            body: AMFValue::ARRAY(body)
        },
    };

    let amfstream = AMFSerializer::serialize_amf_message(message);
    match send_request(method, amfstream) {
        Ok(response) => {
            std::fs::File::create("idk.txt").unwrap().write_all(&response).unwrap();
            return Some(AMFDeserializer::deserialize_amf_message(response));
        },
        Err(err) => {eprintln!("Error: {}", err);
        return None;}
    }
}
fn get_random_number() -> u32 {
    // Use the current time as a seed
    let time_since_epoch = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    let seed = time_since_epoch.as_secs();

    // Simple pseudo-random generation based on the seed (this is not cryptographically secure)
    let random_number = (seed * 123456789) % 100; // Random number between 0 and 99
    random_number as u32
}
pub struct TicketGenerator;
impl TicketGenerator {
    pub fn generate_header(ticket: String) -> AMFValue {
        let random_number: u32 = get_random_number();
        let random_number_str = random_number.to_string();
        let random_number_bytes = random_number_str.as_bytes();
        let md5_hash = hex_encode(&md5::compute(random_number_bytes).to_vec());
        

        let result = format!("{}{}{:?}", ticket, md5_hash, hex_encode(&random_number_bytes.to_vec()));
        let mut map:HashMap<String,AMFValue> = HashMap::new();
        map.insert("Ticket".into(),AMFValue::STRING(result));
        map.insert("anyAttribute".into(),  AMFValue::NULL);
        AMFValue::ASObject(ASObject{
            name:None,
            items: map
        })
    }
}
pub struct ChecksumCalculator;
impl ChecksumCalculator {
    pub fn create_checksum(data: &Vec<AMFValue>) -> String {
        let checksumable = Self::from_array(data) + "2zKzokBI4^26#oiP" + &Self::get_ticket_value(data);
        let mut hasher = Sha1::new();
        hasher.update(checksumable.as_bytes());
       
      return hasher.hexdigest();
    }

    pub fn get_ticket_value(data: &Vec<AMFValue>) -> String {
        for item in data {
            match item{
                AMFValue::ASObject(value)=>{
                    if let AMFValue::STRING(ticket) = value.items.get_key_value("Ticket").unwrap().1{
                let podzial: Vec<&str> = ticket.split(',').collect();
                if let Some(koncowka) = podzial.last() {
                    return format!("{}{}", podzial[0], &koncowka[koncowka.len() - 5..]);
                }}
            }
                _=>continue
            }
        }
        "XSV7%!5!AX2L8@vn".to_string()
    }

    pub fn from_array(data: &Vec<AMFValue>) -> String {
        data.iter()
            .filter_map(|value| {
                if let AMFValue::ASObject(val) = value {
                    if let AMFValue::STRING(ticket) = val.items.get_key_value("Ticket").unwrap().1{
                        return None
                    }
                    None
                } else if let AMFValue::INT(num) = value {
                    Some(num.to_string())
                } else if let AMFValue::STRING(s) = value {
                    Some(s.clone())
                } else if let AMFValue::BOOL(b) = value {
                    Some(if *b { "True".to_string() } else { "False".to_string() })
                } else if let AMFValue::BYTEARRAY(array) = value {
                    Some(Self::from_byte_array(array))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn from_byte_array(data: &Vec<u8>) -> String {
      
        if data.len() <= 20 {
            return hex_encode(data);
        } else {
            let ar: Vec<u8> = (0..20).map(|i| data[data.len() / 20 * i]).collect();
            return hex_encode(&ar);
        }
    
    }
}
pub fn hex_encode(value:&Vec<u8>)->String{
    return value.iter().map(|x|format!("{:02x}",x)).collect::<String>();
}