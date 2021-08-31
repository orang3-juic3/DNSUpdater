mod error;

use error_chain::error_chain;
use std::env;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use crate::error::Errors::ApiError;
use crate::error::Errors;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String>= env::args().collect();
    let client = reqwest::Client::new();
    let target: String;
    let re = Regex::new(r"newip=.+").unwrap();
    if re.is_match(args.last().unwrap()) {
        target = args.last().unwrap().clone().graphemes(true).into_iter().skip(6).collect();
    } else {
        target = ip(&client).await.ok_or_else(|| println!("Crap")).unwrap();
    }
    let mut header_map = HeaderMap::new();
    header_map.insert(HeaderName::from_static("x-auth-email"), HeaderValue::from_str(&args[2]).unwrap());
    header_map.insert(HeaderName::from_static("authorization"), HeaderValue::from_str(&format!(" Bearer {}", &args[3])).unwrap());
    header_map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let zone_id = get_zone_id(&client, &header_map).await.unwrap();
    let items = get_matching_records(&client, &zone_id, &header_map).await.unwrap();
    update_records(&client, items, &header_map, &target).await.unwrap().iter().for_each(|it| println!("{}", it));
    Ok(())
}

async fn ip(client : &Client) -> Option<String> {
    let res = client.get("https://api.ipify.org/").send().await.ok()?;
    return Some(res.text().await.ok()?);
}

#[derive(Deserialize, Debug)]
struct Zone {
    result: Vec<ZoneItem>
}

#[derive(Deserialize, Debug)]
struct ZoneItem {
    id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct ResultSuccess {
    success: bool,
}

async fn get_zone_id(client : &Client, headers: &HeaderMap) -> std::result::Result<String, Errors> {
    let args : Vec<String> = env::args().collect();
    let res = client.get(format!("https://api.cloudflare.com/client/v4/zones?name={}&status=active", args[1]))
                    .headers(headers.clone())
                    .send().await.or(Err(Errors::ApiError))?;
    let zone : Zone = res.json().await.or(Err(ApiError))?;
    Ok(zone.result[0].id.to_string())
}

#[derive(Deserialize, Debug)]
struct PageInfo {
    page : i16,
    total_pages: i16,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DNSRecordItem {
    id : String,
    zone_id: String,
    name : String,
    r#type: String,
    content: String,
    ttl: i32,
    proxied: bool,
}

#[derive(Deserialize, Debug)]
struct DNSRecord {
    result: Vec<DNSRecordItem>,
    result_info: PageInfo,
}


async fn get_matching_records(client : &Client, zone_id: &String, auth_headers: &HeaderMap) -> std::result::Result<Vec<DNSRecordItem>, Errors> {
    let res = client.get(format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records?&per_page=100", zone_id))
        .headers(auth_headers.clone()).send().await.or(Err(Errors::ApiError))?;
    let mut items: Vec<DNSRecordItem> = Vec::new();
    let mut record : DNSRecord = res.json().await.or(Err(ApiError))?;
    items.append(&mut record.result);
    if record.result_info.total_pages > 1 {
        for i in 2..record.result_info.total_pages + 1 {
            let res = client.get(format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records?&per_page=100&page={}", zone_id, i))
                .headers(auth_headers.clone()).send().await.or(Err(Errors::ApiError))?;
            let mut record : DNSRecord = res.json().await.or(Err(ApiError))?;
            items.append(&mut record.result)
        }
    }
    let record_type = &env::args().collect::<Vec<String>>()[4];
    let names = get_target_names();
    return Ok(items.iter().filter(|&it| it.r#type == *record_type && names.contains(&it.name) ).map(|it| it.clone()).collect())
}

fn get_target_names() -> Vec<String> {
    let mut args = env::args().collect::<Vec<String>>()[4..].to_vec();
    let re = Regex::new(r"newip=.+").unwrap();
    if re.is_match(args.last().unwrap()) {
        args.pop();
    }
    return args
}

async fn update_records(client : &Client, records : Vec<DNSRecordItem>, headers: &HeaderMap, target : &String) -> std::result::Result<Vec<String>, Errors> {
    if records.is_empty() {
        return Err(Errors::NoMatchError)
    }
    let mut results: Vec<String> = Vec::new();
    for mut i in records {
        let message = format!("Record with name {}: {} -> {}", i.name, i.content, target.clone());
        i.content = target.clone();
        let res = client.put(format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", i.zone_id, i.id))
            .headers(headers.clone()).body(serde_json::to_vec(&i).unwrap()).send().await.or(Err(Errors::ApiError))?;
        let success : ResultSuccess = res.json().await.or(Err(Errors::ApiError))?;
        if !success.success {
            return Err(Errors::ApiError)
        }


        results.push(message);
    }
    Ok(results)
}






