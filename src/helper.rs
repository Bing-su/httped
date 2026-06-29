use bytes::Bytes;
use data_encoding::BASE64;

pub fn encode_string(input: &Bytes, content_type: Option<&str>) -> String {
    match String::from_utf8(input.to_vec()) {
        Ok(s) => s,
        Err(_) => format!(
            "data:{};base64,{}",
            content_type.unwrap_or("application/octet-stream"),
            BASE64.encode(input)
        ),
    }
}
