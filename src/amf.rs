use native_tls::TlsConnector;


use crate::checksum::ChecksumCalculator;
use crate::amfstuff::AMFSerializer;
use crate::amfstuff::AMFDeserializer;
use crate::amfstuff::AMFMessage;
use crate::amfstuff::AMFBody;
use crate::amfstuff::AMFHeader;
use std::{ any::Any,io::{Write,Read},net::{ SocketAddr, TcpStream, ToSocketAddrs}, time::Duration};
pub fn send_amf(method: &str,body:Vec<Box<dyn Any>>)->Box<dyn Any>{
    let message:AMFMessage = AMFMessage{
        headers: vec![
            AMFHeader{
            name: String::from("needClassName"),
            must_understand: false,
            body: Box::new(false)
            }
        ,
        AMFHeader{
            name: String::from("id"),
            must_understand: false,
            body: Box::new(ChecksumCalculator::create_checksum(&body))
            }],
        version: 0,
        body: AMFBody{
            target:String::from(method),
            response:String::from("/1"),
            body:Box::new(body)
        }
    };
    let amfstream: Vec<u8> = AMFSerializer::serialize_message(&message);
    let connector = TlsConnector::new().unwrap();
    //let addr = SocketAddr::from_str("54.154.22.229:443").unwrap();
    let addr:SocketAddr = "ws-pl.mspapis.com:443".to_socket_addrs().unwrap().next().unwrap();
    let stream = TcpStream::connect_timeout(&addr,Duration::from_secs(3)).expect("There was an error while connecting to host");
    let mut stream = connector.connect("ws-pl.mspapis.com", stream).unwrap();
    let mut final_vec:Vec<u8> = Vec::new();
    final_vec.write(format!("POST /Gateway.aspx?method={} HTTP/1.1\r\n",method).as_bytes()).unwrap();
    final_vec.write("content-type: application/x-amf\r\n".as_bytes()).unwrap();
    final_vec.write("referer: app:/cache/t1.bin/[[DYNAMIC]]/2\r\n".as_bytes()).unwrap();
    final_vec.write("user-agent: Mozilla/5.0 (Windows; U; en) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0\r\n".as_bytes()).unwrap();
    final_vec.write(format!("content-length: {}\r\n",amfstream.len()).as_bytes()).unwrap();
    final_vec.write("connection: close\r\n".as_bytes()).unwrap();
    final_vec.write("host: ws-pl.mspapis.com\r\n\r\n".as_bytes()).unwrap();
    final_vec.write(&amfstream).unwrap();
    
    stream.write_all(&final_vec).expect("Could not write body");
    let mut response = Vec::new();

    stream.read_to_end(&mut response).expect("Could not read stream!");
   // std::fs::File::create("lol").unwrap().write_all(&response).unwrap();
   
   let http_headers_end = response.windows(4).position(|window| window == b"\r\n\r\n").unwrap_or_else(||{
    panic!("Could not read http headers end");
   });
   let body = &response[http_headers_end + 4..];
   if body.len() == 0{
    panic!("Response body is null!");
   }
  // std::fs::File::create("lol").unwrap().write_all(bodybytes).unwrap();
  //  return Box::new(amfstuff::Null);
    return AMFDeserializer::deserialize(body);

}
