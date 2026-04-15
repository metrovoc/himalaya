use std::{net::TcpStream, sync::Arc};

use anyhow::{bail, Result};
#[cfg(feature = "native-tls")]
use native_tls::TlsConnector;
use pimalaya_toolbox::stream::{Stream, Tls, TlsProvider};
#[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
use rustls::{ClientConnection, StreamOwned};
use secrecy::SecretString;

use crate::{account::Account, config::GmailConfig};

pub type GmailAccount = Account<GmailConfig>;

pub const GMAIL_HOST: &str = "gmail.googleapis.com";
pub const GMAIL_PORT: u16 = 443;

#[derive(Debug)]
pub struct GmailSession {
    pub stream: Stream,
    pub http_auth: SecretString,
    pub user_id: String,
    pub tls: Tls,
}

pub fn build_stream(tls: &Tls) -> Result<Stream> {
    let tcp = TcpStream::connect((GMAIL_HOST, GMAIL_PORT))?;

    let stream = match tls.provider()? {
        #[cfg(any(feature = "rustls-aws", feature = "rustls-ring"))]
        TlsProvider::Rustls => {
            let config = tls.build_rustls_client_config()?;
            let server_name = GMAIL_HOST.try_into()?;
            let connection = ClientConnection::new(Arc::new(config), server_name)?;
            Stream::Rustls(StreamOwned::new(connection, tcp))
        }
        #[cfg(feature = "native-tls")]
        TlsProvider::NativeTls => {
            let mut builder = TlsConnector::builder();

            if let Some(pem_path) = &tls.cert {
                let pem = std::fs::read(pem_path)?;
                let cert = native_tls::Certificate::from_pem(&pem)?;
                builder.add_root_certificate(cert);
            }

            let connector = builder.build()?;
            Stream::NativeTls(connector.connect(GMAIL_HOST, tcp)?)
        }
        #[allow(unreachable_patterns)]
        _ => bail!("no TLS provider configured for Gmail"),
    };

    Ok(stream)
}

impl GmailAccount {
    pub fn new_gmail_session(&self) -> Result<GmailSession> {
        let tls: Tls = self.backend.tls.clone().try_into()?;
        let stream = build_stream(&tls)?;
        let http_auth = match &self.backend.auth {
            crate::config::GmailAuthConfig::Bearer { token } => token.clone().get()?,
        };

        Ok(GmailSession {
            stream,
            http_auth,
            user_id: self.backend.user_id.clone(),
            tls,
        })
    }
}

#[macro_export]
macro_rules! gmail_run {
    ($session:expr, $factory:expr, $result_ty:path) => {{
        use ::std::{thread::sleep, time::Duration};

        let _: ::core::option::Option<$result_ty> = None;
        let mut backoff = Duration::from_millis(250);
        let mut attempt = 0u32;

        loop {
            let mut coroutine = ($factory)()?;
            let mut arg = None;

            let attempt_result = loop {
                match coroutine.resume(arg.take()) {
                    ::io_gmail::GmailSendResult::Ok {
                        response,
                        keep_alive,
                    } => {
                        if !keep_alive {
                            $session.stream = $crate::gmail::account::build_stream(&$session.tls)?;
                        }
                        break ::core::result::Result::Ok(response);
                    }
                    ::io_gmail::GmailSendResult::Io { input } => {
                        arg = Some(::io_socket::runtimes::std_stream::handle(
                            &mut $session.stream,
                            input,
                        )?);
                    }
                    ::io_gmail::GmailSendResult::Err { err } => {
                        break ::core::result::Result::Err(err)
                    }
                }
            };

            match attempt_result {
                Ok(response) => break ::anyhow::Result::Ok(response),
                Err(err) => {
                    if err.status() == Some(401) {
                        break ::anyhow::Result::Err(::anyhow::anyhow!(
                            "Gmail returned 401 - refresh your access token (`ortie access-token refresh gmail`) and retry."
                        ));
                    }

                    if err.is_retryable() && attempt < 4 {
                        ::log::warn!(
                            "Gmail transient error: {err}. Retrying in {:?}",
                            backoff
                        );
                        attempt += 1;
                        sleep(backoff);
                        backoff = backoff.saturating_mul(2);
                        $session.stream = $crate::gmail::account::build_stream(&$session.tls)?;
                        continue;
                    }

                    break ::anyhow::Result::Err(err.into());
                }
            }
        }
    }};
}
