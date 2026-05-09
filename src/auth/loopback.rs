use anyhow::{anyhow, bail, Context, Result};
use std::{thread, time::Duration};
use tiny_http::{Response, Server};
use tokio::sync::oneshot;

pub struct LoopbackHandle {
    pub redirect_uri: String,
    pub code_rx: oneshot::Receiver<String>,
    pub shutdown_tx: oneshot::Sender<()>,
}

pub fn spawn(expected_state: String) -> Result<LoopbackHandle> {
    let server = Server::http("127.0.0.1:0")
        .map_err(|error| anyhow!("could not start CLI callback server: {error}"))?;
    let port = server
        .server_addr()
        .to_ip()
        .map(|addr| addr.port())
        .ok_or_else(|| anyhow!("CLI callback server did not bind to a TCP port"))?;
    let redirect_uri = format!("http://localhost:{port}/auth/callback");
    let (code_tx, code_rx) = oneshot::channel();
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    thread::spawn(move || {
        let mut code_tx = Some(code_tx);
        loop {
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            match server.recv_timeout(Duration::from_millis(250)) {
                Ok(Some(request)) => {
                    let result = parse_callback(request.url(), &expected_state);
                    let response = match result {
                        Ok(code) => {
                            if let Some(tx) = code_tx.take() {
                                let _ = tx.send(code);
                            }
                            Response::from_string(success_page()).with_status_code(200)
                        }
                        Err(error) => {
                            Response::from_string(format!("AxiomDB CLI auth failed: {error}"))
                                .with_status_code(400)
                        }
                    };
                    let _ = request.respond(response);
                    break;
                }
                Ok(None) => {}
                Err(_) => break,
            }
        }
    });

    Ok(LoopbackHandle {
        redirect_uri,
        code_rx,
        shutdown_tx,
    })
}

pub fn parse_manual_code(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("authorization code is required");
    }
    if value.starts_with("http://") || value.starts_with("https://") {
        let url = url::Url::parse(value).context("redirect URL is invalid")?;
        return url
            .query_pairs()
            .find_map(|(key, value)| (key == "code").then(|| value.into_owned()))
            .filter(|code| !code.trim().is_empty())
            .ok_or_else(|| anyhow!("redirect URL did not contain a code"));
    }
    Ok(value.to_string())
}

fn parse_callback(path_and_query: &str, expected_state: &str) -> Result<String> {
    let url = url::Url::parse(&format!("http://localhost{path_and_query}"))
        .context("callback URL is invalid")?;
    if url.path() != "/auth/callback" {
        bail!("unexpected callback path");
    }

    let mut code = None;
    let mut state = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            _ => {}
        }
    }
    if state.as_deref() != Some(expected_state) {
        bail!("authorization state mismatch");
    }
    code.filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("authorization code missing"))
}

fn success_page() -> &'static str {
    r#"<!doctype html>
<html>
  <head><title>AxiomDB CLI authenticated</title></head>
  <body style="font-family: Inter, system-ui, sans-serif; background:#050505; color:#f8f8f8; padding:48px;">
    <h1>AxiomDB OAuth complete</h1>
    <p>You can close this tab and return to your terminal.</p>
  </body>
</html>"#
}

#[cfg(test)]
mod tests {
    use super::parse_manual_code;

    #[test]
    fn accepts_code_or_redirect_url() {
        assert_eq!(parse_manual_code("abc").unwrap(), "abc");
        assert_eq!(
            parse_manual_code("http://localhost:49152/auth/callback?code=ac_123&state=s").unwrap(),
            "ac_123"
        );
    }
}
