use std::error::Error;
use std::io::{Read, Write, ErrorKind};
use std::net::TcpStream;
use base64::engine::general_purpose;
use base64::Engine;
use rand::Rng;

// Pomocnicza funkcja do zapisu ramek WS
pub fn write_ws_frame(
    stream: &mut TcpStream,
    opcode: u8,
    payload: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut frame = Vec::new();

    // FIN + opcode
    frame.push(0x80 | opcode);

    let mask_bit = 0x80;
    let len = payload.len();

    if len <= 125 {
        frame.push(mask_bit | len as u8);
    } else if len <= 65535 {
        frame.push(mask_bit | 126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        return Err("WebSocket payload too large".into());
    }

    // Klient MUSI maskować dane w WebSocket
    let mut rng = rand::thread_rng();
    let mask: [u8; 4] = rng.gen();
    frame.extend_from_slice(&mask);

    // Maskowanie payloadu
    for (i, b) in payload.iter().enumerate() {
        frame.push(b ^ mask[i % 4]);
    }

    stream.write_all(&frame)?;
    stream.flush()?; // Wymuszamy wysłanie bufora
    Ok(())
}

pub struct PresenceInstance {
    pub stream: TcpStream,
}

impl PresenceInstance {
    // Zmieniamy na Box<dyn Error>, bo masz wiele typów błędów (IO, UTF8, Custom)
    pub fn connect_socket(profile_id: &str, token: &str) -> Result<Self, Box<dyn Error>> {
        // Pobieramy IP z WinInet
        let presence_raw = unsafe {
            crate::httpwindows::net_connection(
                "presence.mspapis.com",
                "/getServer",
                "GET",
                None,
                None,
                None,
            )
        }.map_err(|_| "Failed to get presence server from WinInet")?;

        let host_dash = String::from_utf8(presence_raw)?;
        let host = host_dash.replace("-", ".");

        // Łączymy się po TCP
        let mut stream = TcpStream::connect(format!("{}:10843", host))?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;

        // === HANDSHAKE ===
        let mut rng = rand::rng();
        let key: [u8; 16] = rng.random();
        let key_b64 = general_purpose::STANDARD.encode(key);

        let req = format!(
            "GET /{host_dash}/?transport=websocket HTTP/1.1\r\n\
             Host: {host}:10843\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: {}\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n",
            key_b64
        );

        stream.write_all(req.as_bytes())?;

        let mut resp = [0u8; 1024];
        let n = stream.read(&mut resp)?;
        let resp_str = String::from_utf8_lossy(&resp[..n]);

        if !resp_str.contains("101") {
            return Err("Handshake failed: Server did not return 101 Switching Protocols".into());
        }

        // === INIT MESSAGE (Socket.io / MSP Presence format) ===
        // Najpierw Socket.io potrzebuje ramki tekstowej (0x1)
        let init_msg = r#"42["10",{"senderProfileId":null,"messageType":10,"messageContent":{"username":"pid","version":1,"country":"PL","access_token":"toks","applicationId":"APPLICATION_WEB"}}]"#
            .replace("pid", profile_id)
            .replace("toks", token);

        write_ws_frame(&mut stream, 0x1, init_msg.as_bytes())?;

        Ok(PresenceInstance { stream })
    }
}