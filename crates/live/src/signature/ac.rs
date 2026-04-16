#[must_use]
pub fn get_ac_signature(site: &str, nonce: &str, user_agent: &str, timestamp: u64) -> String {
    let timestamp_string = timestamp.to_string();
    let seed_value = rolling_xor_hash(site, rolling_xor_hash(&timestamp_string, 0)) % 65_521;

    let payload_bits = signature_payload_bits(timestamp, seed_value);
    let checksum_seed = rolling_xor_hash(&payload_bits.to_string(), 0);
    let encoded_first = encode_signature_chunk(lower_u32(payload_bits >> 2));
    let upper_bits = lower_u32(payload_bits >> 32);
    let encoded_second = encode_signature_chunk(lower_u32(
        (payload_bits << 28) | (u64::from(upper_bits) >> 4),
    ));
    let xor_bits = 0x22B1_EC98_u64 ^ payload_bits;
    let encoded_third =
        encode_signature_chunk(lower_u32((u64::from(upper_bits) << 26) | (xor_bits >> 6)));
    let tail_char = signature_char(lower_u32(xor_bits & 63));

    let nonce_mix = ((rolling_xor_hash(user_agent, checksum_seed) % 65_521) << 16)
        | (rolling_xor_hash(nonce, checksum_seed) % 65_521);
    let encoded_nonce = encode_signature_chunk(nonce_mix >> 2);
    let encoded_tail = encode_signature_chunk(
        (nonce_mix << 28) | lower_u32((0x0008_0120_u64 ^ payload_bits) >> 4),
    );
    let encoded_seed = encode_signature_chunk(seed_value);

    let signature_body = format!(
        "{SIGNATURE_HEAD}{encoded_first}{encoded_second}{encoded_third}{tail_char}{encoded_nonce}{encoded_tail}{encoded_seed}"
    );

    format!("{signature_body}{}", signature_suffix(&signature_body))
}

const SIGNATURE_HEAD: &str = "_02B4Z6wo00f01";

fn lower_u32(value: u64) -> u32 {
    u32::try_from(value & u64::from(u32::MAX)).expect("value is masked to u32 range")
}

fn rolling_xor_hash(input: &str, seed: u32) -> u32 {
    let mut value = seed;
    for ch in input.chars() {
        value = (value ^ (ch as u32)).wrapping_mul(65_599);
    }
    value
}

fn rolling_add_hash(input: &str, seed: u32) -> u32 {
    let mut value = seed;
    for ch in input.chars() {
        value = value.wrapping_mul(65_599).wrapping_add(ch as u32);
    }
    value
}

fn signature_payload_bits(timestamp: u64, seed_value: u32) -> u64 {
    let mixed = timestamp ^ (u64::from(seed_value) * 65_521);
    let mut binary = format!("{mixed:b}");
    if binary.len() < 32 {
        binary = format!("{binary:0>32}");
    }

    u64::from_str_radix(&format!("10000000110000{binary}"), 2).unwrap_or(0)
}

fn encode_signature_chunk(value: u32) -> String {
    let mut output = String::with_capacity(5);
    for shift in [24_u32, 18, 12, 6, 0] {
        let bits = (value >> shift) & 63;
        output.push(signature_char(bits));
    }
    output
}

fn signature_char(code: u32) -> char {
    match code {
        0..=25 => char::from_u32(code + 65).unwrap_or('A'),
        26..=51 => char::from_u32(code + 71).unwrap_or('a'),
        52..=61 => char::from_u32(code - 4).unwrap_or('0'),
        _ => char::from_u32(code - 17).unwrap_or('.'),
    }
}

fn signature_suffix(signature_body: &str) -> String {
    let hex = format!("{:x}", rolling_add_hash(signature_body, 0));
    if hex.len() >= 2 {
        hex[hex.len() - 2..].to_string()
    } else {
        format!("{hex:0>2}")
    }
}

#[cfg(test)]
mod tests {
    use super::get_ac_signature;

    #[test]
    fn matches_python_reference() {
        let actual = get_ac_signature("www.douyin.com/", "test_nonce", "test_ua", 1_700_000_000);
        assert_eq!(actual, "_02B4Z6wo00f01HR4CBQAAIDA.r-6dHnd-dB0WAyAAHhV51");
    }
}
