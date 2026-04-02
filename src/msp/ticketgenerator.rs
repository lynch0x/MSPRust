use std::{collections::BTreeMap, sync::atomic::{AtomicU32, Ordering}};
use rand::Rng;
use crate::amfnew::{AMFValue, ASObject};


pub fn generate_header(ticket: Option<&str>, token: Option<&str>) -> AMFValue {
    // 1. Obsługa Ticketu: jeśli None, użyj domyślnego
    let base_ticket = ticket.unwrap_or("4e4b5fbbbb602b6d35bea8460aa8f8e5353632");
let mut r = rand::rng();
    // 2. Generowanie MarkingID (MD5 + Hex)
    let current_num:i32 =r.random_range(0..=1000);
    let num_str = current_num.to_string();
    let md5_hash = hex::encode(md5::compute(num_str.as_bytes()).0);
    let num_hex = hex::encode(num_str.as_bytes());

    // Finalny string ticketu: base + markingID
    let ticket_result = match ticket{
        Some(base_ticket)=>format!("{}{}{}", base_ticket, md5_hash, num_hex),
        None=>format!("{}{}",  md5_hash, num_hex)
    };

    // 3. Budowanie struktury anyAttribute (Token)
    let mut any_attr_map: BTreeMap<String, AMFValue> = BTreeMap::new();

    // Jeśli token to Some -> String, jeśli None -> NULL
    let token_value = match token {
        Some(t) => AMFValue::STRING(t.to_string()),
        None => AMFValue::NULL,
    };
    any_attr_map.insert("Token".into(), token_value);

    // 4. Składanie głównego obiektu Header
    let mut main_map: BTreeMap<String, AMFValue> = BTreeMap::new();
    main_map.insert("anyAttribute".into(), AMFValue::ASObject(ASObject {
        name: None,
        items: any_attr_map,
    }));
    main_map.insert("Ticket".into(), AMFValue::STRING(ticket_result));


    AMFValue::ASObject(ASObject {
        name: None,
        items: main_map,
    })
}