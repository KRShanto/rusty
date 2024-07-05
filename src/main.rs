use dotenv::dotenv;
use hyper::{body::Buf, header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::io::{stdin, stdout, Write};

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIChoiceMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct OpenAIChoices {
    message: OpenAIChoiceMessage,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoices>,
}

#[derive(Serialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIChoiceMessage>,
}

const URI: &str = "https://api.openai.com/v1/chat/completions";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv().ok();

    // Check for environment variable OPENAI_API_KEY
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Error: missing environment variable OPENAI_API_KEY");
            std::process::exit(1);
        }
    };

    // take the model from env, if not provided use the default
    let model = match env::var("OPENAI_MODEL") {
        Ok(model) => model,
        Err(_) => String::from("gpt-3.5-turbo"),
    };

    let https = HttpsConnector::new();
    let client = Client::builder().build(https);

    let user_input = env::args().skip(1).collect::<Vec<String>>().join(" ");

    // If no arguments were provided, ask for user input
    let user_input = if user_input.is_empty() {
        print!("Enter prompt: ");
        let _ = stdout().flush();
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("Failed to read prompt.");
        input
    } else {
        user_input
    };

    let auth_header_val = format!("Bearer {}", api_key);

    let openai_request = OpenAIRequest {
        model,
        messages: vec![
            OpenAIChoiceMessage {
                role: "system".to_string(),
                content: String::from("You are bash command generator. Only return the command."),
            },
            OpenAIChoiceMessage {
                role: "user".to_string(),
                content: String::from("How to list contents of a directory in bash?"),
            },
            OpenAIChoiceMessage {
                role: "assistant".to_string(),
                content: "ls".to_string(),
            },
            OpenAIChoiceMessage {
                role: "user".to_string(),
                content: user_input.trim().to_string(),
            },
        ],
    };

    let body = Body::from(serde_json::to_vec(&openai_request)?);

    let req = Request::post(URI)
        .header(header::CONTENT_TYPE, "application/json")
        .header("Authorization", &auth_header_val)
        .body(body)
        .unwrap();

    let res = client.request(req).await?;

    let body = hyper::body::aggregate(res).await?;

    let json: OpenAIResponse = match serde_json::from_reader(body.reader()) {
        Ok(response) => response,
        Err(e) => {
            println!("Error: check environment variable OPENAI_API_KEY or try again later");
            println!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let bash = json.choices[0]
        .message
        .content
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    println!("{}", bash);

    Ok(())
}
