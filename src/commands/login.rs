use std::path::Path;
use std::{collections::HashMap, fs};

use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json;
use tiny_http::{Response, Server};

#[derive(Serialize, Deserialize)]
struct ClientSecret {
    client_id: String,
    project_id: String,
    auth_uri: String,
    token_uri: String,
    auth_provider_x509_cert_url: String,
    client_secret: String,
    redirect_uris: Vec<String>,
}

async fn get_tokens(client_secret: &ClientSecret, auth_code: &str) {
    let http_client = Client::new();

    let mut req_body = HashMap::new();
    req_body.insert("client_id", client_secret.client_id.as_str());
    req_body.insert("client_secret", &client_secret.client_secret.as_str());
    req_body.insert("code", auth_code);
    req_body.insert("grant_type", "authorization_code");
    req_body.insert("redirect_uri", client_secret.redirect_uris[0].as_str());

    let response = http_client
        .post(&client_secret.token_uri)
        .form(&req_body)
        .send()
        .await;
    match response {
        Ok(response) => {
            if response.status().is_success() {
                let tokens = response.text().await;
                match tokens {
                    Ok(tokens) => {
                        println!("Tokens received: {}", tokens);
                    }
                    Err(e) => eprintln!("Failed to read response text: {}", e),
                }
            } else {
                eprintln!("Failed to get tokens: {}", response.status());
            }
        },
        Err(e) => {
            eprintln!("Failed to send request: {}", e);
        }
    }
}

pub async fn login() {
    let client_secret: ClientSecret =
        serde_json::from_str(&fs::read_to_string(Path::new("client_secret.json")).unwrap())
            .unwrap();

    let mut auth_url = Url::parse(&client_secret.auth_uri).expect("Invalid auth URI");
    auth_url.query_pairs_mut()
        .append_pair("client_id", &client_secret.client_id)
        .append_pair("redirect_uri", &client_secret.redirect_uris[0])
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/gmail.readonly")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");

    println!("Please open the following URL in your browser to log in:");
    println!("{}", auth_url);

    let server = match Server::http("localhost:8080") {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Failed to start http server: {}", e);
            return;
        }
    };

    println!("OAuth callback server localhost:8080");

    for req in server.incoming_requests() {
        let url = match Url::parse(&format!("http://dumy/{}", String::from(req.url()))) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Failed to parse response url: {}", e);
                return;
            }
        };

        let response =
            Response::from_string("You may close this window and return to the application.")
                .with_header(tiny_http::Header {
                    field: "Content-Type".parse().unwrap(),
                    value: "text/html; charset=UTF-8".parse().unwrap(),
                });

        if let Err(e) = req.respond(response) {
            eprintln!("Failed to send response: {}", e);
        }

        let mut auth_code = None;
        let mut error = None;

        for pair in url.query_pairs() {
            match pair.0.as_ref() {
                "code" => auth_code = Some(pair.1.to_string()),
                "error" => error = Some(pair.1.to_string()),
                _ => {}
            }
        }

        if error.is_some() {
            eprintln!("OAuth error: {}", error.unwrap());
            return;
        }

        if let Some(code) = auth_code {
            println!("Authorization code received");
            get_tokens(&client_secret, &code).await;
        }
    }
}
