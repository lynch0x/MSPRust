use rayon::iter::ParallelIterator;
mod msp;
mod amfnew;
mod nebula;
pub mod httpwindows;

use std::{thread::sleep, time::Duration};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use base64::Engine;
use rand::Rng;
use amfnew::{AMFError, AMFValue};
use nebula::*;
use crate::amfnew::ASObject;

fn get_user_input(message: &str) -> String {
    println!("{}", message);
    let mut input_buffer = String::new();
    std::io::stdin().read_line(&mut input_buffer).expect("incorrect string");
    input_buffer.trim().to_string()
}

fn actorid_from_ticket(ticket: &str) -> f64 {
    ticket.split(',')
        .nth(1)
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn login_do(login: String, password: String,token:&str,proxy:Option<&str>) -> Result<String, AMFError> {


    let result = msp::client::send_amf(
        "MovieStarPlanet.WebService.User.AMFUserServiceWeb.Login",
        AMFValue::ARRAY(vec![
            generate_header(None,Some(token)),
            AMFValue::STRING(login.clone()),
            AMFValue::STRING(password),
            AMFValue::ARRAY(vec![AMFValue::INT(206970446f64),AMFValue::INT(902894417f64),AMFValue::INT(1240523696f64),AMFValue::INT(-1588257202f64)]),
            AMFValue::NULL,
            AMFValue::NULL,
            AMFValue::STRING("MSP1-Standalone:XXXXXX".into()),
        ]),proxy
    )?;
println!("{:?}",result);

    let login_status = result.as_object().items.get("loginStatus")
        .ok_or(AMFError::ValueNotFound)?;
    let status = login_status.as_object().items.get("status")
        .ok_or(AMFError::ValueNotFound)?
        .as_string();

    if status != "Success" {
        println!("LoginStatus: {}",status);
        return Err(AMFError::RequestFailed);
    }
    let actor = login_status.as_object().items.get("actor").ok_or(AMFError::ValueNotFound)?.as_object();
    let level = actor.items.get("Level").ok_or(AMFError::ValueNotFound)?.as_int();
    if level < 6f64{
        return Err(AMFError::RequestFailed);
    }
    let ticket = login_status.as_object().items.get("ticket")
        .ok_or(AMFError::ValueNotFound)?
        .as_string();
    Ok(String::from(ticket))
}

pub fn load_proxies(path: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let proxies = reader
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    println!("{:?}",proxies);
    Ok(proxies)
}
pub fn load_accounts(path: &str) -> Result<Vec<(String, String, String)>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut accounts = Vec::new();

    for (line_no, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();

        // pomijamy puste / komentarze
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();


                let ticket = parts[0];
                let pid = parts[1];
              //  let token = parts[2];

                if ticket.is_empty() || pid.is_empty()  {
                    eprintln!("accounts.txt:{} -> nieprawidłowe dane", line_no + 1);
                    continue;
                }

                accounts.push((
                    ticket.to_string(),
                    pid.to_string(),
                    String::new()
                ));




    }

    Ok(accounts)
}
fn get_actor_by_name(name:String,proxy:&str)->Result<f64,AMFError>
{
    Ok(msp::client::send_amf("MovieStarPlanet.WebService.AMFActorService.GetActorIdByName", AMFValue::ARRAY(vec![
        AMFValue::STRING(name)
    ]),None)?.as_int())
}
fn put_actor_rel(vec:&mut Vec<AMFValue>,id:f64){
    let mut rel_obj:BTreeMap<String,AMFValue> = BTreeMap::new();
    rel_obj.insert("ActorId".into(),AMFValue::INT(id));
    vec.push(AMFValue::ASObject(ASObject{
        name:None,
        items:rel_obj
    }));
}
fn create_movie(ticket:&str, token:&str) -> Result<f64, AMFError> {
    let mut bajty = Vec::with_capacity(2);
    bajty.push(0);
    bajty.push(0);

    let result = msp::client::send_amf(
        "MovieStarPlanet.MobileServices.AMFMovieService.CreateMovieWithSnapshot",
        AMFValue::ARRAY(vec![
            generate_header(Some(ticket),Some(token)),
            AMFValue::STRING("Sek".into()),
            AMFValue::BOOL(false),
            AMFValue::INT(3000f64),

            AMFValue::BYTEARRAY(bajty.clone()),
            AMFValue::BYTEARRAY(bajty.clone()),
            AMFValue::ARRAY(vec![
                AMFValue::INT(84771709f64),
                AMFValue::INT(1181110f64),
                AMFValue::INT(73874187f64),
                AMFValue::INT(88161038f64)

            ]),
            AMFValue::BYTEARRAY(bajty.clone()),
            AMFValue::BYTEARRAY(bajty)
        ]),None
    )?;

let movie_id=result.as_object().items.get("movieId").ok_or(AMFError::ValueNotFound)?.as_int();
    if movie_id == -1f64{
        return Err(AMFError::RequestFailed);
    }
    Ok(movie_id)
}
fn watch_movie(ticket:&str, token:&str,id:f64,proxy:Option<&str>) -> Result<(), AMFError> {

    let result = msp::client::send_amf(
        "MovieStarPlanet.MobileServices.AMFMovieService.MovieWatched",
        AMFValue::ARRAY(vec![
            generate_header(Some(ticket),Some(token)),

            AMFValue::INT(id),


        ]),proxy
    )?;

    let awarded_fame = result.as_object().items.get("awardedFame").ok_or(AMFError::ValueNotFound)?.as_int();
    if awarded_fame >10f64{
        return Ok(());
    }
    Err(AMFError::RequestFailed)
}
fn rate_movie(ticket:&str, token:&str,id:f64,proxy:Option<&str>) -> Result<(), AMFError> {
    let mut rng = rand::rng();
    let rating = rng.random_range(1..=5);
    let result = msp::client::send_amf(
        "MovieStarPlanet.MobileServices.AMFMovieService.RateMovie",
        AMFValue::ARRAY(vec![
            generate_header(Some(ticket), Some(token)),
            AMFValue::INT(id),
            AMFValue::INT(rating as f64)
        ]), proxy
    )?;


    Ok(())
}
fn delete_movie(ticket:&str, token:&str,id:f64) -> Result<(), AMFError> {

    let result = msp::client::send_amf(
        "MovieStarPlanet.WebService.MovieService.AMFMovieService.DeleteMovie",
        AMFValue::ARRAY(vec![
            generate_header(Some(ticket),Some(token)),

            AMFValue::INT(id),
            AMFValue::INT(ticket.split(',').nth(1).unwrap().parse::<f64>().unwrap())

        ]),None
    )?;


    Ok(())
}
fn load_summary(ticket:&str, token:&str,id:f64) -> Result<(), AMFError> {

    let result = msp::client::send_amf(
        "MovieStarPlanet.WebService.Profile.AMFProfileService.LoadProfileSummary",
        AMFValue::ARRAY(vec![
            generate_header(Some(ticket),Some(token)),

            AMFValue::INT(id),
            AMFValue::INT(ticket.split(',').nth(1).unwrap().parse::<f64>().unwrap())

        ]),None
    )?;


    Ok(())
}
fn post_login(ticket:&str, token:&str) -> Result<(), AMFError> {

    let _ = msp::client::send_amf(
        "MovieStarPlanet.WebService.ActorService.AMFActorServiceForWeb.GetPostLoginBundleStandalone",
        AMFValue::ARRAY(vec![
            generate_header(Some(ticket),Some(token)),

            AMFValue::INT(ticket.split(',').nth(1).unwrap().parse::<f64>().unwrap())


        ]),None
    )?;


    Ok(())
}
fn spin_wheel(ticket:&str, token:&str) -> Result<(), AMFError> {

    let _ = msp::client::send_amf(
        "MovieStarPlanet.WebService.Awarding.AMFAwardingService.claimDailyAward",
        AMFValue::ARRAY(vec![
            generate_header(Some(ticket),Some(token)),
            AMFValue::STRING("wheel".into()),
            AMFValue::INT(120f64),
            AMFValue::INT(ticket.split(',').nth(1).unwrap().parse::<f64>().unwrap())


        ]),None
    )?;


    Ok(())
}
use rand::seq::IndexedRandom;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator};
use crate::msp::client::send_amf;
use crate::msp::presence::PresenceInstance;
use crate::msp::ticketgenerator::generate_header;
fn base(){
    send_amf("MovieStarPlanet.WebService.AppSettings.AMFAppSettingsService.GetAppSettings",AMFValue::ARRAY(vec![
        generate_header(None,None),
        AMFValue::ARRAY(vec![])
    ]),None);
    std::thread::sleep(Duration::from_secs(2));
    let mut ctx:BTreeMap<String,AMFValue> = BTreeMap::new();
    ctx.insert("OperationSystemVersion".into(),AMFValue::STRING("10".into()));
    ctx.insert("OperationSystemType".into(),AMFValue::STRING("Windows 10".into()));
    ctx.insert("DeviceID".into(),AMFValue::STRING("7e527e77819c90b579de80816d29402f".into()));
    ctx.insert("DeviceModel".into(),AMFValue::STRING("Windows".into()));
    ctx.insert("DeviceManufacturer".into(),AMFValue::STRING("Adobe Windows".into()));
    let mut i:BTreeMap<String,AMFValue> = BTreeMap::new();
    i.insert("Name".into(),AMFValue::STRING("new_user_web".into()));
    i.insert("StepNumber".into(),AMFValue::INT(0f64));
    i.insert("StepName".into(),AMFValue::STRING("game_open".into()));
    i.insert("Platform".into(),AMFValue::STRING("pc".into()));
    i.insert("DesktopContext".into(),AMFValue::ASObject(ASObject{
        name:None,
        items:ctx
    }));
    send_amf("MovieStarPlanet.WebService.Analytics.AMFAnalyticsService.FunnelEventAnonymous",AMFValue::ARRAY(vec![

        AMFValue::ASObject(ASObject{
            name:None,
            items:i
        })
    ]),None);
    std::thread::sleep(Duration::from_secs(2));
}
fn main() {
    let bots = load_accounts("bots.txt").expect("Błąd ładowania kont");
    let proxies = load_proxies("proxies.txt").expect("Błąd ładowania proxy");

    let proxy_index = Arc::new(AtomicUsize::new(0));
    let proxies = Arc::new(proxies);

  //  let lr = nebula::login(String::from("PL"), "9pata9".into(), "".into(), None, true);

//    if lr.login_state == LoginState::LOGGEDIN {
 //       if let Ok(lr1) = login_do("9pata9".into(), "zaq1@WSXlol".into(), &lr.access_token, None) {

 //           if let Ok(movie_id) = create_movie(&lr1, &lr.access_token) {

                bots.par_iter().for_each(|bot| {

                    let (u, p, x) = bot;

                    // 🔁 ROTACJA PROXY
                    let idx = proxy_index.fetch_add(1, Ordering::Relaxed);
                    let proxy = &proxies[idx % proxies.len()];

                    let lr = nebula::login(
                        String::from("PL"),
                        u.clone(),
                        p.clone(),
                        Some(&proxy), 
                        true
                    );
	//	println!("{:?}",lr);
                    if lr.login_state == LoginState::LOGGEDIN {
                        base();
                        if let Ok(lr1) = login_do(u.clone(), p.clone(), &lr.access_token, None) {

                            if let Ok(_) = PresenceInstance::connect_socket(&lr.profile_id, &lr.access_token) {
                       //         let _ = post_login(&lr1, &lr.access_token);
 //let _ = spin_wheel(&lr1, &lr.access_token);
                               // let _ = load_summary(&lr1, &lr.access_token, 1181110f64);
                                if let Ok(result) = send_amf("MovieStarPlanet.WebService.UserSession.AMFUserSessionService.GiveAutographAndCalculateTimestamp",AMFValue::ARRAY(vec![
                                    generate_header(Some(&lr1),Some(&lr.access_token)),
                                    AMFValue::INT(lr1.split(',').nth(1).unwrap().parse::<f64>().unwrap()),
                                    AMFValue::INT(1181110f64)
                                ]),Some(&proxy)){
                                    println!("{:?}",result);
                                }

                               // if let Ok(_) = watch_movie(&lr1, &lr.access_token, movie_id,Some(&proxy)) {
                               //     let _ = rate_movie(&lr1, &lr.access_token, movie_id,Some(&proxy));
                               //     println!("Movie watched using proxy: {}", proxy);
                               // }
                            }
                        }
                    }
                });
          //  }
     //   }
  //  }
}