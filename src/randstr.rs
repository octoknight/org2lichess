use rand::Rng;

pub fn random_string() -> Result<String, std::str::Utf8Error> {
    let mut rng = rand::rng();
    let mut oauth_state_bytes: [u8; 64] = [0; 64];
    rng.fill_bytes(&mut oauth_state_bytes);
    for i in 0..oauth_state_bytes.len() {
        oauth_state_bytes[i] = (oauth_state_bytes[i] % 26) + 97;
    }
    Ok(std::str::from_utf8(&oauth_state_bytes)?.to_string())
}
