use oauth2::PkceCodeChallenge;
use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, Clone)]
pub struct PkceState {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

impl PkceState {
    pub fn new() -> Self {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        Self {
            verifier: verifier.secret().to_string(),
            challenge: challenge.as_str().to_string(),
            state: random_token(32),
        }
    }
}

fn random_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::PkceState;

    #[test]
    fn creates_pkce_material() {
        let state = PkceState::new();
        assert!(state.verifier.len() >= 32);
        assert!(state.challenge.len() >= 32);
        assert_eq!(state.state.len(), 32);
    }
}
