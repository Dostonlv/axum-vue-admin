use crate::config::smtp::SmtpConfig;
use crate::errors::app_error::AppError;
use crate::internal_server_error;
use anyhow::Result;
use core::str::FromStr;
use lettre::{
    message::header::ContentType, message::Mailbox, transport::smtp::authentication::Credentials,
    Address, AsyncSmtpTransport, AsyncTransport, Message, SmtpTransport, Tokio1Executor, Transport,
};
use tracing::{info, instrument};

enum Mailer {
    Secure(AsyncSmtpTransport<Tokio1Executor>),
    Insecure(SmtpTransport),
}

pub struct EmailService {
    mailer: Mailer,
}

impl EmailService {
    pub fn new(config: &SmtpConfig) -> Result<Self, AppError> {
        let mailer = match config.tls {
            // --- Encrypted connection (async) ---
            true => {
                let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
                    &config.host,
                )
                .map_err(|e| internal_server_error!("Failed to create STARTTLS relay: {}", e))?;
                builder = builder.port(config.port);
                if let (Some(username), Some(password)) = (&config.username, &config.password) {
                    if !username.is_empty() {
                        let creds = Credentials::new(username.clone(), password.clone());
                        builder = builder.credentials(creds);
                    }
                }
                Mailer::Secure(builder.build())
            }
            // --- Plain connection (sync) ---
            false => {
                let mut builder = SmtpTransport::builder_dangerous(&config.host);
                builder = builder.port(config.port);
                if let (Some(username), Some(password)) = (&config.username, &config.password) {
                    if !username.is_empty() {
                        let creds = Credentials::new(username.clone(), password.clone());
                        builder = builder.credentials(creds);
                    }
                }
                Mailer::Insecure(builder.build())
            }
        };

        Ok(Self { mailer })
    }

    // Send an email
    #[instrument(skip(self, to, subject, body))]
    pub async fn send(
        &self,
        from: &str,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), AppError> {
        let address = Address::from_str(from)
            .map_err(|e| internal_server_error!("Invalid sender address [{}]: {}", from, e))?;
        let mailbox = Mailbox::new(Some("AxumVueAdmin".to_string()), address);
        let recipient = to
            .parse()
            .map_err(|e| internal_server_error!("Invalid recipient address [{}]: {}", to, e))?;
        let email = Message::builder()
            .from(mailbox)
            .to(recipient)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(String::from(body))
            .map_err(|e| internal_server_error!("Failed to build email message: {}", e))?;

        info!(recipient = %to, "Sending email...");

        return match &self.mailer {
            Mailer::Secure(mailer) => {
                mailer.send(email).await?;
                info!("Email sent");
                Ok(())
            }
            Mailer::Insecure(mailer) => {
                let mailer_clone = mailer.clone();
                tokio::task::spawn_blocking(move || mailer_clone.send(&email)).await??; // first ? handles JoinError, second handles SmtpError
                info!("Email sent");
                Ok(())
            }
        };
    }
}
