use std::path::Path;
use std::{collections::HashMap, fs};

use keyring::Entry;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json;
use tiny_http::{Response, Server};

use crate::utils::config::{UserConfig, get_config_file_path, create_config_file, load_user_config, ConfigError};

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


#[derive(Serialize, Deserialize)]
struct Token {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
    token_type: String,
    refresh_token_expires_in: u64,
}

async fn get_tokens(client_secret: &ClientSecret, auth_code: &str) {
    // Send a POST request to exchange the authorization code for tokens
    
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
                        println!("Response: {}", tokens);
                        let token: Token = serde_json::from_str(&tokens).unwrap();
                        println!("Access Token: {}", token.access_token);
                        println!("Refresh Token: {}", token.refresh_token);

                        // Store user info in a config file
                        let user_info_response = http_client
                            .get("https://www.googleapis.com/oauth2/v1/userinfo")
                            .bearer_auth(&token.access_token)
                            .send()
                            .await;
                        match user_info_response {
                            Ok(user_info_response) => {
                                if user_info_response.status().is_success() {
                                    let user_info_text = user_info_response.text().await.unwrap();
                                    let user_info: UserConfig = serde_json::from_str(&user_info_text).unwrap();

                                    // Create or update the user config file
                                    if let Some(config_path) = get_config_file_path() {
                                        if !config_path.exists() {
                                            create_config_file();
                                        }
                                        let mut user_config = match load_user_config() {
                                            Ok(config) => config,
                                            Err(ConfigError::NoConfigDirectory) => {
                                                eprintln!("No config directory found, creating one.");
                                                create_config_file();
                                                UserConfig { email: String::new() }
                                            }
                                            Err(ConfigError::ConfigFileNotFound(path)) => {
                                                eprintln!("Config file not found at: {}", path.display());
                                                UserConfig { email: String::new() }
                                            }
                                            Err(ConfigError::ConfigFileReadError(e)) => {
                                                eprintln!("Failed to read config file: {}", e);
                                                UserConfig { email: String::new() }
                                            }
                                            _ => {
                                                eprintln!("An unexpected error occurred while loading user config.");
                                                UserConfig { email: String::new() }
                                            }
                                        };
                                        user_config.email = user_info.email;
                                        fs::write(config_path, toml::to_string(&user_config).unwrap())
                                            .expect("Failed to write user config file");
                                        println!("User config updated successfully.");
                                    } else {
                                        eprintln!("Could not find config file path.");
                                    }

                                    
                                } else {
                                    eprintln!("Failed to get user info: {}", user_info_response.status());
                                }
                            }
                            Err(e) => eprintln!("Failed to send user info request: {}", e),
                        }

                        // Store tokens using keyring
                        let access_entry = Entry::new("pigeon", "access_token").unwrap();
                        access_entry.set_password(&token.access_token).unwrap();

                        let refresh_entry = Entry::new("pigeon", "refresh_token").unwrap();
                        refresh_entry.set_password(&token.refresh_token).unwrap();

                        println!("Tokens stored successfully.");
                    }
                    Err(e) => eprintln!("Failed to read response text: {}", e),
                }
            } else {
                eprintln!("Failed to get tokens: {}", response.status());
            }
        }
        Err(e) => {
            eprintln!("Failed to send request: {}", e);
        }
    }
}

pub async fn login() {
    let client_secret: ClientSecret =
        serde_json::from_str(&fs::read_to_string(Path::new("client_secret.json")).unwrap())
            .unwrap();

    // Construct the authorization URL

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

    // Start a local HTTP server to handle the OAuth callback

    println!("Starting OAuth callback server at localhost:8080");

    let server = match Server::http("localhost:8080") {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Failed to start http server: {}", e);
            return;
        }
    };

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

        break;
    }
}
