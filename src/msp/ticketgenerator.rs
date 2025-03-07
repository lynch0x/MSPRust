use std::{collections::BTreeMap, sync::atomic::{AtomicU32,Ordering}};

use crate::amfnew::{AMFValue, ASObject};
static NUM: AtomicU32 = AtomicU32::new(0);
pub fn generate_header(ticket: String) -> AMFValue {
 
    let random_number = NUM.fetch_add(1, Ordering::SeqCst);
    let random_number_string: String = random_number.to_string();
    let random_number_bytes: &[u8] = random_number_string.as_bytes();
    let md5_hash: String = hex::encode(&md5::compute(random_number_bytes).to_vec());
    

    let result: String = format!("{}{}{}", ticket, md5_hash, hex::encode(random_number_bytes));
    let mut map:BTreeMap<String,AMFValue> = BTreeMap::new();
    map.insert("Ticket".into(),AMFValue::STRING(result));
    map.insert("anyAttribute".into(),  AMFValue::NULL);
    AMFValue::ASObject(ASObject{
        name:None,
        items: map
    })
}