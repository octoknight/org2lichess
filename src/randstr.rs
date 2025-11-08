use rand::Rng;

pub fn random_string() -> Result<String, std::str::Utf8Error> {
    let mut rng = rand::rng();
    let mut oauth_state_bytes: [u8; 64] = [0; 64];
    for x in &mut oauth_state_bytes {
        *x = (rng.random::<u8>() % 26) + 97;
    }
    Ok(std::str::from_utf8(&oauth_state_bytes)?.to_string())
}
