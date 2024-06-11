use structopt::StructOpt;
use reqwest::Client;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio;
use serde::Deserialize;
use std::io::Write;


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
}

#[derive(Serialize)]
struct Payload {
    data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    is_encrypted: bool,
    expire: u64,
}

async fn send_data(data: String, id: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let payload = Payload {
        data,
        id,
        is_encrypted: false,
        expire: 604800,
    };

    let res = client.post("https://api.stagb.in/dev/content")
        .json(&payload)
        .send()
        .await?;

    let status = res.status();
    if status.is_success() { // Check for successful status codes (200s)
        let response_text = res.text().await?;
        // Assuming the response contains the ID in JSON format
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


async fn retrieve_data(id: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.stagb.in/dev/content/{}", id);
    let response = reqwest::get(&url).await?;
    let data_items: Vec<DataItem> = response.json().await?;

    for item in data_items {
        if item.id == id {
            let mut file = File::create(output_path)?;
            file.write_all(item.data.as_bytes())?;
            println!("File saved to {:?}", output_path);
            return Ok(());
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

        send_data(data_content, opt.id).await.unwrap();
    } else if opt.retrieve {
        if let Some(id) = opt.id {
            let output_path = opt.output.unwrap_or_else(|| {
                let default_filename = format!("{}.txt", id);
                std::path::PathBuf::from(default_filename)
            });
            retrieve_data(&id, &output_path).await.unwrap();
        } else {
            eprintln!("ID is required for retrieval.");
            std::process::exit(1);
        }
    } else {
        eprintln!("Invalid action. Use -s to send data or -r to retrieve data.");
        std::process::exit(1);
    }
}
