use std::{any::Any, collections::HashMap, io::{Read, Write}, net::{SocketAddr, TcpStream, ToSocketAddrs}, sync::Arc, time::Duration, vec};


use webpki_roots::TLS_SERVER_ROOTS;
use rand::Rng;
use md5;
use hex;
use rustls::{ClientConfig, ClientConnection, ProtocolVersion, RootCertStore, ServerName};
use sha1::{Sha1,Digest};
pub fn send_amf(method: &str, body: Vec<Box<dyn Any>>) -> Box<dyn Any> {
    let message = AMFMessage {
        headers: vec![
            AMFHeader {
                name: String::from("needClassName"),
                must_understand: false,
                body: Box::new(false),
            },
            AMFHeader {
                name: String::from("id"),
                must_understand: false,
                body: Box::new(ChecksumCalculator::create_checksum(&body)),
            },
        ],
        version: 0,
        body: AMFBody {
            target: String::from(method),
            response: String::from("/1"),
            body: Box::new(body),
        },
    };

    let amfstream = AMFSerializer::serialize_message(&message);
    let mut root_store = RootCertStore::empty();


    root_store.add_trust_anchors(TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
   
    let addr: SocketAddr = "ws-pl.mspapis.com:443"
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    // Configure the client with custom settings
    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
   
    // Customize the cipher suites and ALPN protocols to affect the JA3 fingerprint
    //config. = vec![&rustls::CipherSuite::TLS13_AES_128_GCM_SHA256,&rustls::CipherSuite::TLS13_AES_256_GCM_SHA384];
    config.alpn_protocols.push(b"http/1.0".to_vec());
    let server_name = ServerName::try_from("ws-pl.mspapis.com").unwrap();
    let mut connector = ClientConnection::new(Arc::new(config),server_name).unwrap();
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
        .expect("There was an error while connecting to host");
    stream.set_nonblocking(false).unwrap();
    let mut tls_stream = rustls::Stream::new(&mut connector, &mut stream);
  

    let mut final_vec = Vec::new();
    final_vec
        .write(format!("POST /Gateway.aspx?method={} HTTP/1.0\r\n", method).as_bytes())
        .unwrap();
    final_vec
        .write("content-type: application/x-amf\r\n".as_bytes())
        .unwrap();
    final_vec
        .write("referer: app:/cache/t1.bin/[[DYNAMIC]]/2\r\n".as_bytes())
        .unwrap();
    final_vec
        .write(
            "user-agent: Mozilla/5.0 (Windows; U; en) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0\r\n"
                .as_bytes(),
        )
        .unwrap();
    final_vec
        .write(format!("content-length: {}\r\n", amfstream.len()).as_bytes())
        .unwrap();
    final_vec
        .write("connection: close\r\n".as_bytes())
        .unwrap();
    final_vec
        .write("host: ws-pl.mspapis.com\r\n\r\n".as_bytes())
        .unwrap();
    final_vec.write(&amfstream).unwrap();

    tls_stream.write_all(&final_vec).expect("Could not write body");
        tls_stream.flush().unwrap();
    let mut response = Vec::new();
    connector
        .reader().read(&mut response).unwrap();
        println!("{}",String::from_utf8(response.clone()).unwrap());
    let http_headers_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap_or_else(|| panic!("Could not read http headers end"));

    let body = &response[http_headers_end + 4..];
    if body.is_empty() {
        panic!("Response body is null!");
    }

    AMFDeserializer::deserialize(body)
}

pub struct  AMFHeader{
    pub name: String,
    pub must_understand: bool,
    pub body: Box<dyn Any>
}
pub struct AMFBody{
    pub target:String,
    pub response:String,
    pub body:Box<dyn Any>
}
pub struct AMFMessage{
    pub headers: Vec<AMFHeader>,
    pub body: AMFBody,
    pub version: u16
}
pub struct  AMFSerializer;
impl AMFSerializer {
    pub fn serialize_message(message: &AMFMessage) -> Vec<u8> {
        let mut stream = Vec::new();
        Self::write_uint16(&mut stream, 0);
        Self::write_uint16(&mut stream, message.headers.len() as u16);
        for header in &message.headers {
            Self::write_utf8(&mut stream, &header.name);
            Self::write_boolean(&mut stream, header.must_understand);
            Self::write_int32(&mut stream, -1);
            Self::write_data(&mut stream, &header.body);
        }
        Self::write_uint16(&mut stream, 1);
        Self::write_utf8(&mut stream, &message.body.target);
        Self::write_utf8(&mut stream, &message.body.response);
        Self::write_int32(&mut stream, -1);
        Self::write_data(&mut stream, &message.body.body);
        stream
    }

    fn write_end_markup(stream: &mut Vec<u8>) {
        stream.extend_from_slice(&[0, 0, 9]);
    }

    fn write_data(stream: &mut Vec<u8>, value: &Box<dyn Any>) {
        if let Some(array) = value.downcast_ref::<Vec<Box<dyn Any>>>() {
            Self::write_array(stream, array);
        } else if value.is::<Null>() {
            Self::write_null(stream);
        } else if let Some(s) = value.downcast_ref::<String>() {
            stream.push(2);
            Self::write_utf8(stream, s);
        } else if let Some(s) = value.downcast_ref::<&str>() {
            stream.push(2);
            Self::write_utf8_str(stream, s);
        } else if let Some(b) = value.downcast_ref::<bool>() {
            stream.push(1);
            Self::write_boolean(stream, *b);
        } else if let Some(i) = value.downcast_ref::<i32>() {
            stream.push(0);
            Self::write_double(stream, *i as f64);
        } else if let Some(header) = value.downcast_ref::<TicketHeader>() {
            stream.push(3);
            Self::write_utf8(stream, &"Ticket".to_string());
            stream.push(2);
            Self::write_utf8(stream, &header.ticket);
            Self::write_utf8(stream, &"anyAttribute".to_string());
            Self::write_null(stream);
            Self::write_end_markup(stream);
        } else if let Some(array) = value.downcast_ref::<Vec<u8>>() {
            stream.push(17); // change AMF encoding to AMF3
            stream.push(12); // byte array marker
            let mut len = (array.len() as u32) << 1 | 1;
            Self::write_amf3_uint32_data(stream, &mut len);
            stream.extend_from_slice(array);
        }
    }

    fn write_amf3_uint32_data(stream: &mut Vec<u8>, value: &mut u32) {
        *value &= 536870911;
        if *value < 128 {
            stream.push(*value as u8);
            return;
        }
        if *value < 16384 {
            stream.push(((*value >> 7 & 127) | 128) as u8);
            stream.push((*value & 127) as u8);
            return;
        }
        if *value < 2097152 {
            stream.push(((*value >> 14 & 127) | 128) as u8);
            stream.push(((*value >> 7 & 127) | 128) as u8);
            stream.push((*value & 127) as u8);
            return;
        }
        // TODO implement larger int
    }

    fn write_array(stream: &mut Vec<u8>, value: &Vec<Box<dyn Any>>) {
        stream.push(10);
        Self::write_int32(stream, value.len() as i32);
        for item in value {
            Self::write_data(stream, item);
        }
    }

    fn write_double(stream: &mut Vec<u8>, value: f64) {
        Self::write_bytes(stream, &value.to_be_bytes());
    }

    fn write_null(stream: &mut Vec<u8>) {
        stream.push(5);
    }

    fn write_int32(stream: &mut Vec<u8>, value: i32) {
        Self::write_bytes(stream, &value.to_be_bytes());
    }

    fn write_boolean(stream: &mut Vec<u8>, boolean: bool) {
        stream.push(boolean as u8);
    }

    fn write_uint16(stream: &mut Vec<u8>, value: u16) {
        Self::write_bytes(stream, &value.to_be_bytes());
    }

    fn write_utf8(stream: &mut Vec<u8>, value: &String) {
        let bytes = value.as_bytes();
        Self::write_uint16(stream, bytes.len() as u16);
        Self::write_bytes(stream, bytes);
    }

    fn write_utf8_str(stream: &mut Vec<u8>, value: &str) {
        let bytes = value.as_bytes();
        Self::write_uint16(stream, bytes.len() as u16);
        Self::write_bytes(stream, bytes);
    }

    fn write_bytes(stream: &mut Vec<u8>, bytes: &[u8]) {
        stream.extend_from_slice(bytes);
    }
}

pub struct Null;

pub struct AMFDeserializer;

impl AMFDeserializer {
    pub fn deserialize(bytes: &[u8]) -> Box<dyn Any> {
        let mut pos: usize = 2;
        let headers_length = Self::read_uint16(bytes, &mut pos);
        for _ in 0..headers_length {
            Self::read_string(bytes, &mut pos);
            Self::read_boolean(bytes, &mut pos);
            Self::read_int32(bytes, &mut pos);
            Self::read_data(bytes, &mut pos);
        }
        Self::read_uint16(bytes, &mut pos);
        Self::read_string(bytes, &mut pos);
        Self::read_string(bytes, &mut pos);
        Self::read_int32(bytes, &mut pos);
        Self::read_data(bytes, &mut pos)
    }

    pub fn read_data(bytes: &[u8], pos: &mut usize) -> Box<dyn Any> {
        let marker = bytes[*pos];
        *pos += 1;
        Self::read_data_from_marker(bytes, pos, marker)
    }

    pub fn read_data_from_marker(bytes: &[u8], pos: &mut usize, marker: u8) -> Box<dyn Any> {
        match marker {
            0 => Box::new(Self::read_double(bytes, pos)),
            1 => Box::new(Self::read_boolean(bytes, pos)),
            2 => Box::new(Self::read_string(bytes, pos)),
            3 => Box::new(Self::read_aso(bytes, pos)),
            5 => Box::new(Null),
            8 => Box::new(Self::read_associative_array(bytes, pos)),
            10 => Box::new(Self::read_array(bytes, pos)),
            11 => {
                *pos += 10;
                Box::new(Duration::new(0, 0))
            }
            16 => Box::new(Self::read_object(bytes, pos)),
            _ => panic!("unknown type at {}", *pos),
        }
    }

    pub fn read_object(bytes: &[u8], pos: &mut usize) -> HashMap<String, Box<dyn Any>> {
        Self::read_string(bytes, pos);
        Self::read_aso(bytes, pos)
    }

    pub fn read_array(bytes: &[u8], pos: &mut usize) -> Vec<Box<dyn Any>> {
        let length = Self::read_int32(bytes, pos);
        (0..length).map(|_| Self::read_data(bytes, pos)).collect()
    }

    pub fn read_associative_array(bytes: &[u8], pos: &mut usize) -> HashMap<String, Box<dyn Any>> {
        *pos += 4;
        Self::read_aso(bytes, pos)
    }

    pub fn read_int32(bytes: &[u8], pos: &mut usize) -> i32 {
        let bytes = Self::read_bytes::<4>(bytes, pos);
        i32::from_be_bytes(bytes)
    }

    pub fn read_aso(bytes: &[u8], pos: &mut usize) -> HashMap<String, Box<dyn Any>> {
        let mut map = HashMap::new();
        loop {
            let name = Self::read_string(bytes, pos);
            let marker = bytes[*pos];
            *pos += 1;
            if marker == 9 {
                break;
            }
            map.insert(name, Self::read_data_from_marker(bytes, pos, marker));
        }
        map
    }

    pub fn read_uint16(bytes: &[u8], pos: &mut usize) -> u16 {
        let bytes = Self::read_bytes::<2>(bytes, pos);
        u16::from_be_bytes(bytes)
    }

    pub fn read_string(bytes: &[u8], pos: &mut usize) -> String {
        let len = Self::read_uint16(bytes, pos);
        let bytes = Self::read_bytes_vec(bytes, pos, len as usize);
        String::from_utf8(bytes).unwrap()
    }

    pub fn read_boolean(bytes: &[u8], pos: &mut usize) -> bool {
        let boolean = bytes[*pos] != 0;
        *pos += 1;
        boolean
    }

    pub fn read_double(bytes: &[u8], pos: &mut usize) -> f64 {
        let array = Self::read_bytes::<8>(bytes, pos);
        f64::from_be_bytes(array)
    }

    pub fn read_bytes<const N: usize>(bytes: &[u8], pos: &mut usize) -> [u8; N] {
        let mut buffer = [0; N];
        buffer.copy_from_slice(&bytes[*pos..*pos + N]);
        *pos += N;
        buffer
    }

    pub fn read_bytes_vec(bytes: &[u8], pos: &mut usize, len: usize) -> Vec<u8> {
        let bytes = &bytes[*pos..*pos + len];
        *pos += len;
        bytes.to_vec()
    }
}





pub struct ChecksumCalculator;
impl ChecksumCalculator {
    pub fn create_checksum(data: &Vec<Box<dyn Any>>) -> String {
        let checksumable = Self::from_array(data) + "2zKzokBI4^26#oiP" + &Self::get_ticket_value(data);
        let mut hasher = Sha1::new();
        hasher.update(checksumable);
        hex::encode(hasher.finalize())
    }

    pub fn get_ticket_value(data: &Vec<Box<dyn Any>>) -> String {
        for item in data {
            if let Some(ticketheader) = item.downcast_ref::<TicketHeader>() {
                let podzial: Vec<&str> = ticketheader.ticket.split(',').collect();
                if let Some(koncowka) = podzial.last() {
                    return format!("{}{}", podzial[0], &koncowka[koncowka.len() - 5..]);
                }
            }
        }
        "XSV7%!5!AX2L8@vn".to_string()
    }

    pub fn from_array(data: &Vec<Box<dyn Any>>) -> String {
        data.iter()
            .filter_map(|value| {
                if value.is::<TicketHeader>() {
                    None
                } else if let Some(num) = value.downcast_ref::<i32>() {
                    Some(num.to_string())
                } else if let Some(s) = value.downcast_ref::<String>() {
                    Some(s.clone())
                } else if let Some(s) = value.downcast_ref::<&str>() {
                    Some(s.to_string())
                } else if let Some(b) = value.downcast_ref::<bool>() {
                    Some(if *b { "True".to_string() } else { "False".to_string() })
                } else if let Some(array) = value.downcast_ref::<Vec<u8>>() {
                    Some(Self::from_byte_array(array))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn from_byte_array(data: &Vec<u8>) -> String {
        if data.len() <= 20 {
            hex::encode(data)
        } else {
            let ar: Vec<u8> = (0..20).map(|i| data[data.len() / 20 * i]).collect();
            hex::encode(ar)
        }
    }
}
pub struct TicketHeader {
    pub ticket: String,
}

pub struct TicketGenerator;

impl TicketGenerator {
    pub fn generate_header(ticket: String) -> String {
        let random_number: u16 = rand::thread_rng().gen_range(0..1000);
        let random_number_str = random_number.to_string();
        let random_number_bytes = random_number_str.as_bytes();
        let md5_hash = format!("{:x}", md5::compute(random_number_bytes));
        let hex_encoded = hex::encode(random_number_bytes);

        format!("{}{}{}", ticket, md5_hash, hex_encoded)
    }
}