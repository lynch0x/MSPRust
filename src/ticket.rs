
use rand::Rng;
use crate::amfserializer::Null;
use hex;
pub struct TicketHeader{
    pub ticket:String,
    pub any_attribute:Null
}
pub struct TicketGenerator;

impl TicketGenerator{
    pub fn generate_header(ticket:String)->String{
        let random_number:String = rand::thread_rng().gen_range(0..1000).to_string();
        let random_number_string_bytes = random_number.as_bytes();
        let fina = ticket.to_string() + &format!("{:x}", md5::compute(random_number_string_bytes)) +&hex::encode(random_number_string_bytes);
        return fina;
    }
}