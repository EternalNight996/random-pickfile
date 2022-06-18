#[test]
fn encode() {
    use p_utils::base64::encode;
    assert!("aGVsbG8gd29ybGR+" == encode(b"hello world~"));
}
#[test]
fn decode() {
    use p_utils::base64::decode;
    assert!(Ok(b"hello world~".to_vec()) == decode(b"aGVsbG8gd29ybGR+"));
}
 