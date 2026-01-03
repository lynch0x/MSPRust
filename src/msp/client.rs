use crate::amfnew::*;
use super::checksumcalculator;
use windows::core::PCSTR;
use windows::Win32::{Foundation::*, Networking::WinInet::*};
use std::ffi::CString;

static mut H_INTERNET: *mut ::core::ffi::c_void = std::ptr::null_mut();
static mut H_CONNECT: *mut ::core::ffi::c_void = std::ptr::null_mut();

pub unsafe fn net_connection(
    host: &str,
    path: &str,
    data: &[u8],
) -> Result<Vec<u8>, ()> {

    if H_INTERNET.is_null() {
        let agent = CString::new(
            "Mozilla/5.0 (Windows; U; pl-PL) AppleWebKit/533.19.4 \
             (KHTML, like Gecko) AdobeAIR/32.0"
        ).unwrap();

        H_INTERNET = InternetOpenA(
            PCSTR(agent.as_ptr() as _),
            0u32,
            PCSTR::null(),
            PCSTR::null(),
            0,
        );

        if H_INTERNET.is_null() {
            return Err(());
        }
    }

    if H_CONNECT.is_null() {
        let host_c = CString::new(host).unwrap();

        H_CONNECT = InternetConnectA(
            H_INTERNET,
            PCSTR(host_c.as_ptr() as _),
            443,
            PCSTR::null(),
            PCSTR::null(),
            INTERNET_SERVICE_HTTP,
            0,
            None,
        );

        if H_CONNECT.is_null() {
            return Err(());
        }
    }

    let path_c = CString::new(path).unwrap();
    let method = CString::new("POST").unwrap();
    let referer=CString::new("app:/cache/t1.bin/[[DYNAMIC]]/2").unwrap();
    let version=CString::new("HTTP/1.1").unwrap();
    let h_request = HttpOpenRequestA(
        H_CONNECT,
        PCSTR(method.as_ptr() as _),
        PCSTR(path_c.as_ptr() as _),
        PCSTR(version.as_ptr() as _),
        PCSTR(referer.as_ptr() as _),
        None,
        INTERNET_FLAG_SECURE | INTERNET_FLAG_KEEP_CONNECTION | INTERNET_FLAG_NO_CACHE_WRITE,
        None,
    );

    if h_request.is_null() {
        return Err(());
    }

    let headers = CString::new("Content-Type: application/x-amf\r\n").unwrap();

    let _ = HttpSendRequestA(
        h_request,
        Some(headers.as_bytes()),
        Some(data.as_ptr() as _),
        data.len() as _,
    );

    let mut buffer = Vec::new();
    let mut temp = [0u8; 1024];
    let mut read = 0;

    loop {
        if !InternetReadFile(
            h_request,
            temp.as_mut_ptr() as _,
            temp.len() as u32,
            &mut read,
        ).is_ok() || read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read as usize]);
    }

    let _ = InternetCloseHandle(h_request);

    Ok(buffer)
}

pub fn send_amf(method: &str, body: AMFValue) -> Result<AMFValue, AMFError> {
    let message = AMFMessage {
        headers: vec![
            AMFHeader {
                name: String::from("needClassName"),
                must_understand: false,
                body: AMFValue::BOOL(false),
            },
            AMFHeader {
                name: String::from("id"),
                must_understand: false,
                body: AMFValue::STRING(checksumcalculator::create_checksum(&body))
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

    let resp = unsafe {
        net_connection("ws-pl.mspapis.com", &path, amf_stream.as_slice())
    }.map_err(|_| AMFError::RequestFailed)?;

    AMFDeserializer::deserialize_amf_message(resp)
}