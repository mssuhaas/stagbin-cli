use structopt::StructOpt;
use reqwest::Client;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use tokio;
use serde::Deserialize;
use std::io::Write;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct DataItem {
    ttl: Option<u64>,
    id: String,
    url: Option<bool>,
    data: String,
}


#[derive(StructOpt, Debug)]
#[structopt(name = "stagbin")]
struct Opt {
    #[structopt(short = "s", long, conflicts_with = "retrieve")]
    send: bool,

    #[structopt(short = "r", long, conflicts_with = "send")]
    retrieve: bool,

    #[structopt(short, long, required_if("send", "true"))]
    data: Option<String>,

    #[structopt(short, long, required_if("send", "true"))]
    id: Option<String>,

    #[structopt(short, long, parse(from_os_str))]
    file: Option<std::path::PathBuf>,

    #[structopt(short, long, parse(from_os_str))]
    output: Option<std::path::PathBuf>,

    #[structopt(short = "e", long, default_value = "7d")]
    expire: String,

}

#[derive(Serialize)]
struct Payload {
    data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    is_encrypted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    expire: Option<u64>,
}


fn parse_expire(expire: &str) -> Option<u64> {
    if expire == "n" {
        return None;
    }

    let (num, unit) = expire.split_at(expire.len() - 1);
    let num: u64 = match num.parse() {
        Ok(n) => n,
        Err(_) => return Some(604800), // Default to 7 days if parsing fails
    };

    match unit {
        "d" => Some(num * 86400),
        "w" => Some(num * 7 * 86400),
        "m" => Some(num * 30 * 86400), // Approximating a month to 30 days
        _ => Some(604800), // Default to 7 days for invalid units
    }
}


async fn send_data(data: String, id: Option<String>, expire: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let payload = Payload {
        data,
        id,
        is_encrypted: false,
        expire,
    };

    let res = client.post("https://api.stagb.in/dev/content")
        .json(&payload)
        .send()
        .await?;

    let status = res.status();
    if status.is_success() {
        let response_text = res.text().await?;
        if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if let Some(id_value) = json_data.get("id") {
                if let Some(id_str) = id_value.as_str() {
                    println!("URL: https://stagb.in/{}", id_str);
                } else {
                    eprintln!("Failed to extract ID from response");
                }
            } else {
                eprintln!("ID not found in response");
            }
        } else {
            eprintln!("Failed to parse response as JSON");
        }
    } else {
        eprintln!("Error sending data: {}", status);
    }

    Ok(())
}

async fn retrieve_data(id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://api.stagb.in/dev/content/{}", id);
    let response = reqwest::get(&url).await?;
    let data_items: Vec<DataItem> = response.json().await?;

    for item in data_items {
        if item.id == id {
            return Ok(item.data);
        }
    }

    Err("Item with the specified ID not found".into())
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    if opt.send {
        let mut data_content = opt.data.unwrap_or_else(String::new);

        if let Some(file_path) = opt.file {
            let mut file = File::open(file_path).expect("Unable to open file");
            file.read_to_string(&mut data_content).expect("Unable to read file");
        }

        if data_content.is_empty() {
            eprintln!("No data provided. Use --data or --file to specify data.");
            std::process::exit(1);
        }

        let expire = parse_expire(&opt.expire);
        send_data(data_content, opt.id, expire).await.unwrap();
    } else if opt.retrieve {
        let id = opt.id.unwrap_or_else(|| {
            eprintln!("ID is required for retrieval. Use -i or --id to specify the ID.");
            std::process::exit(1);
        });

        match retrieve_data(&id).await {
            Ok(content) => {
                if let Some(output_path) = opt.output {
                    let mut file = File::create(&output_path).expect("Unable to create file");
                    file.write_all(content.as_bytes()).expect("Unable to write to file");
                    println!("File saved to {:?}", output_path);
                } else {
                    println!("{}", content);
                }
            },
            Err(e) => {
                eprintln!("Error retrieving data: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Invalid action. Use -s to send data or -r to retrieve data.");
        std::process::exit(1);
    }
}