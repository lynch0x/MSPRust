use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uchar};
use crate::amfnew::*;

use super::checksumcalculator;

#[link(name = "mspclient", kind = "static")]
extern "C" {
    fn NetConnection(
        host: *const c_char,
        path: *const c_char,
        data: *const c_uchar,
        length: c_int,
        response_length: *mut c_int,
    ) -> *const c_uchar;
    fn free_response(ptr: *const c_uchar);
}

fn call_net_connection(host: &str, path: &str, data: &[u8]) -> Result<Vec<u8>,()> {
    let c_host = CString::new(host).expect("CString conversion failed");
    let c_path = CString::new(path).expect("CString conversion failed");

    let mut response_length: c_int = 0;

    unsafe {
        let response_ptr = NetConnection(
            c_host.as_ptr(),
            c_path.as_ptr(),
            data.as_ptr(),
            data.len() as c_int,
            &mut response_length as *mut c_int,
        );

        if response_ptr.is_null() || response_length <= 0 {
            return Err(());
        }

        let response_slice = std::slice::from_raw_parts(response_ptr, response_length as usize);
        let result = response_slice.to_vec();
        free_response(response_ptr);
        Ok(result)
    }
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
                body: AMFValue::STRING(checksumcalculator::create_checksum(&body))
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
    let mut path = String::new();
    path += "/Gateway.aspx?method=";
    path += method;
    match call_net_connection("ws-pl.mspapis.com",path.as_str(), &amfstream) {
        Ok(response) => {
            return Some(AMFDeserializer::deserialize_amf_message(response));
        },
        Err(_) => {eprintln!("Error while TLS!");
        return None;}
    }
}