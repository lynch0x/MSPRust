use windows::core::PCSTR;
use windows::Win32::{Foundation::*, Networking::WinInet::*};
use std::ffi::CString;




fn parse_proxy(raw_proxy: &str) -> (CString, CString, CString) {
    let raw_proxy = raw_proxy.trim();



    let url_without_prefix = raw_proxy.trim_start_matches("http://");
    let parts: Vec<&str> = url_without_prefix.split('@').collect();



    let auth_part = parts[0];
    let addr_part = parts[1];

    let auth_split: Vec<&str> = auth_part.split(':').collect();

    let user = auth_split[0];
    let pass = auth_split[1];

    let proxy_addr = CString::new(format!("http://{}", addr_part)).unwrap();
    let proxy_user = CString::new(user).unwrap();
    let proxy_pass = CString::new(pass).unwrap();

    (proxy_addr, proxy_user, proxy_pass)
}
pub unsafe fn net_connection(
    host: &str,
    path: &str,
    method:&str,
    headers: Option<&[u8]>,
    data: Option<&[u8]>,proxy:Option<&str>
) -> Result<Vec<u8>, ()> {
    if proxy.is_none(){
        return net_connection_no_proxy(host,path,method,headers,data);
    }
    let (proxy_addr, proxy_user, proxy_pass) = parse_proxy(proxy.unwrap());

    let agent = CString::new(
            "Mozilla/5.0 (Windows; U; en) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0"
        ).unwrap();

    let h_internet = InternetOpenA(
        PCSTR(agent.as_ptr() as _),
        3u32, // Używamy proxy
        PCSTR(proxy_addr.as_ptr() as _),
        PCSTR::null(),
        0,
    );

        if h_internet.is_null() {
            return Err(());
        }

    let _ = InternetSetOptionA(
        Some(h_internet),
        INTERNET_OPTION_PROXY_USERNAME,
        Some(proxy_user.as_ptr() as _),
        proxy_user.to_bytes().len() as u32,
    );
    let _ = InternetSetOptionA(
        Some( h_internet),
        INTERNET_OPTION_PROXY_PASSWORD,
        Some(proxy_pass.as_ptr() as _),
        proxy_pass.to_bytes().len() as u32,
    );

        let host_c = CString::new(host).unwrap();

        let h_connect = InternetConnectA(
            h_internet,
            PCSTR(host_c.as_ptr() as _),
            443,
            PCSTR(proxy_user.as_ptr() as _),
            PCSTR(proxy_pass.as_ptr() as _),
            INTERNET_SERVICE_HTTP,
            0,
            None,
        );

        if h_connect.is_null() {
            return Err(());
        }


    let path_c = CString::new(path).unwrap();
    let method = CString::new(method).unwrap();
    let referer=CString::new("app:/cache/t1.bin/[[DYNAMIC]]/2").unwrap();
    let version=CString::new("HTTP/1.1").unwrap();

    let h_request = HttpOpenRequestA(
        h_connect,
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
  //  let proxy_auth_header = CString::new("Proxy-Authorization: Basic YXdxa2x4cTpxdm91NGN4ejdvNnY=\r\n").unwrap();
  //  let _ = HttpAddRequestHeadersA(
  //      h_request,
 //       proxy_auth_header.as_bytes(),
  //      HTTP_ADDREQ_FLAG_ADD | HTTP_ADDREQ_FLAG_REPLACE,
  //  );
    let _ = InternetSetOptionA(
        Some(h_request),
        INTERNET_OPTION_PROXY_USERNAME,
        Some(proxy_user.as_ptr() as _),
        proxy_user.to_bytes().len() as u32,
    );
    let _ = InternetSetOptionA(
        Some( h_request),
        INTERNET_OPTION_PROXY_PASSWORD,
        Some(proxy_pass.as_ptr() as _),
        proxy_pass.to_bytes().len() as u32,
    );
    let _ = HttpSendRequestA(
        h_request,
        match headers{
            Some(val)=>Some(val),
            None=>None
        },
        match data{
            Some(val)=>Some(val.as_ptr() as _),
            None=>None
        },
        match data{
            Some(val)=>val.len() as u32,
            None=>0u32
        }

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
   let _ = InternetCloseHandle(h_connect);
    let _ = InternetCloseHandle(h_internet);
    Ok(buffer)
}


pub unsafe fn net_connection_no_proxy(
    host: &str,
    path: &str,
    method:&str,
    headers: Option<&[u8]>,
    data: Option<&[u8]>
) -> Result<Vec<u8>, ()> {


    let agent = CString::new("Mozilla/5.0 (Windows; U; en) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0").unwrap();


        let h_internet = InternetOpenA(
            PCSTR(agent.as_ptr() as _),
            0u32,
            PCSTR::null(),
            PCSTR::null(),
            0,
        );

        if h_internet.is_null() {
            return Err(());
        }



        let host_c = CString::new(host).unwrap();

        let h_connect = InternetConnectA(
            h_internet,
            PCSTR(host_c.as_ptr() as _),
            443,
            PCSTR::null(),
            PCSTR::null(),
            INTERNET_SERVICE_HTTP,
            0,
            None,
        );

        if h_connect.is_null() {
            return Err(());
        }




    let path_c = CString::new(path).unwrap();
    let method = CString::new(method).unwrap();
    let referer=CString::new("app:/cache/t1.bin/[[DYNAMIC]]/2").unwrap();

    let h_request = HttpOpenRequestA(
        h_connect,
        PCSTR(method.as_ptr() as _),
        PCSTR(path_c.as_ptr() as _),
        PCSTR::null(),
        PCSTR(referer.as_ptr() as _),
        None,
        INTERNET_FLAG_SECURE   ,
        None,
    );

    if h_request.is_null() {
        return Err(());
    }

    let _ = HttpSendRequestA(
        h_request,
        match headers{
            Some(val)=>Some(val),
            None=>None
        },
        match data{
            Some(val)=>Some(val.as_ptr() as _),
            None=>None
        },
        match data{
            Some(val)=>val.len() as u32,
            None=>0u32
        }

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

    let _ = InternetCloseHandle(h_request);  let _ = InternetCloseHandle(h_connect);  let _ = InternetCloseHandle(h_internet);
    Ok(buffer)
}
