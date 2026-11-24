// Ed25519 signatures from the publisher's key (private key only on the publishing PC,
// ignored/market/update_signing_key.pem; public key built in): agent builds and remote commands.
use ring::signature::{ED25519, Ed25519KeyPair, KeyPair, UnparsedPublicKey};

/// Ops the agent takes unsigned (all Discord can send); everything else must be signed.
pub const UNSIGNED: [&str; 3] = ["status", "start", "pause"];

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap_or(0))
        .collect()
}

pub fn verify(msg: &str, sig: &str) -> bool {
    let key = unhex(include_str!("../update_key.hex").trim());
    UnparsedPublicKey::new(&ED25519, key)
        .verify(msg.as_bytes(), &unhex(sig))
        .is_ok()
}

/// Signs with the private key in the data folder; panics where it is missing.
pub fn sign(msg: &str) -> String {
    use base64::Engine;
    let pem =
        std::fs::read_to_string(crate::assets::data().join("update_signing_key.pem")).unwrap();
    let b64: String = pem.lines().filter(|l| !l.starts_with("-----")).collect();
    let der = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .unwrap();
    let key = Ed25519KeyPair::from_pkcs8_maybe_unchecked(&der).unwrap();
    assert_eq!(
        hex(key.public_key().as_ref()),
        include_str!("../update_key.hex").trim()
    );
    hex(key.sign(msg.as_bytes()).as_ref())
}

/// A remote command for `machine`, signed, with a counter (unix ms) the agent takes only once.
pub fn command(machine: &str, mut cmd: serde_json::Value) -> serde_json::Value {
    if UNSIGNED.contains(&cmd["op"].as_str().unwrap_or("")) {
        return cmd;
    }
    cmd["n"] = (chrono::Utc::now().timestamp_millis() as u64).into();
    let signed = cmd.to_string();
    let sig = sign(&format!("hf-market-cmd:{machine}:{signed}"));
    serde_json::json!({"op": cmd["op"], "signed": signed, "sig": sig})
}
