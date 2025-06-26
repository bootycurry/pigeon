use imap;
use keyring;
use native_tls::TlsConnector;

use crate::utils::config::{UserConfig, load_user_config};

struct GmailOAuth2 {
    user_email: String,
    access_token: String,
}

impl imap::Authenticator for GmailOAuth2 {
    type Response = String;
    #[allow(unused_variables)]
    fn process(&self, data: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user_email, self.access_token
        )
    }
}

pub fn get_inbox() {
    let tls = match TlsConnector::builder().build() {
        Ok(tls) => tls,
        Err(e) => {
            eprintln!("Failed to create TLS connector: {}", e);
            return;
        }
    };

    let client = match imap::connect(("imap.gmail.com", 993), "imap.gmail.com", &tls) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to IMAP server: {}", e);
            return;
        }
    };

    let user_config: UserConfig = match load_user_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load user configuration: {:?}", e);
            return;
        }
    };

    let access_token = match keyring::Entry::new("pigeon", "access_token") {
        Ok(entry) => match entry.get_password() {
            Ok(token) => token,
            Err(e) => {
                eprintln!("Failed to retrieve access token: {}", e);
                return;
            }
        },
        Err(e) => {
            eprintln!("Failed to create keyring entry: {}", e);
            return;
        }
    };

    let gmail_oauth2 = GmailOAuth2 {
        user_email: user_config.email.clone(),
        access_token,
    };

    let mut imap_session = match client.authenticate("XOAUTH2",&gmail_oauth2) {
        Ok(session) => session,
        Err(e) => {
            eprintln!("Failed to authenticate with IMAP server: {:?}", e);
            return;
        }
    };
    let inbox = match imap_session.select("INBOX") {
        Ok(inbox) => inbox,
        Err(e) => {
            eprintln!("Failed to select INBOX: {}", e);
            return;
        }
    };

    println!("Selected INBOX: {:?}", inbox);

    let messages = match imap_session.fetch("1:*", "ENVELOPE") {
        Ok(messages) => messages,
        Err(e) => {
            eprintln!("Failed to fetch messages: {}", e);
            return;
        }
    };

    for message in messages.iter() {
    if let Some(envelope) = message.envelope() {
        let from = envelope.from
            .as_ref()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.mailbox.as_ref())
            .and_then(|mailbox| std::str::from_utf8(mailbox).ok())
            .unwrap_or("Unknown");

        let subject = envelope.subject
            .as_ref()
            .and_then(|s| std::str::from_utf8(s).ok())
            .unwrap_or("No Subject");


        println!("From: {}, Subject: {}", from, subject);
    }
}
}
