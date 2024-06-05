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
    let addr:SocketAddr = "ws-pl.mspapis.com:80".to_socket_addrs().unwrap().next().unwrap();
    let mut tcpclient = TcpStream::connect_timeout(&addr,Duration::from_secs(3)).expect("There was an error while connecting to host");
    let mut final_vec:Vec<u8> = Vec::new();
    final_vec.write(format!("POST /Gateway.aspx?method={} HTTP/1.1\r\n",method).as_bytes()).unwrap();
    final_vec.write("content-type: application/x-amf\r\n".as_bytes()).unwrap();
    final_vec.write("referer: app:/cache/t1.bin/[[DYNAMIC]]/2\r\n".as_bytes()).unwrap();
    final_vec.write("user-agent: Mozilla/5.0 (Windows; U; en) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0\r\n".as_bytes()).unwrap();
    final_vec.write(format!("content-length: {}\r\n",amfstream.len()).as_bytes()).unwrap();
    final_vec.write("connection: close\r\n".as_bytes()).unwrap();
    final_vec.write("host: ws-pl.mspapis.com\r\n\r\n".as_bytes()).unwrap();
    final_vec.write(&amfstream).unwrap();
    tcpclient.write_all(&final_vec).expect("Could not write body");
    let mut response = Vec::new();

tcpclient.read_to_end(&mut response).expect("Could not read stream!");

    let bodybytes =response.split(|x| *x==u8::MAX).last().unwrap();
    return AMFDeserializer::deserialize(bodybytes);

}