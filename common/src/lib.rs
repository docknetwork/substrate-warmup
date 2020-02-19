use sr_primitives::AccountId32;
use substrate_primitives::Public;

/// convert a 0x prefixed hex string to a 32 byte public key
pub fn parse_pubkey<T: Public>(imp: &str) -> Result<T, &'static str> {
    Ok(Public::from_slice(&parse_key_256(imp)?))
}

/// convert a 0x prefixed hex string to an AccountId32
pub fn parse_accountid32(imp: &str) -> Result<AccountId32, &'static str> {
    parse_key_256(imp).map(Into::into)
}

/// panics if slice is wrong length
pub fn slice_to_arr32<T: Default + Copy>(src: &[T]) -> [T; 32] {
    let mut ret: [T; 32] = Default::default();
    ret.copy_from_slice(src);
    ret
}

/// parse a 256 bit, 32 byte key from a 0x prefixed hex string
pub fn parse_key_256(imp: &str) -> Result<[u8; 32], &'static str> {
    let imp: &[u8] = imp.as_bytes();

    // check key is 0x prefixed, remove prefix
    let imp: &[u8] = if imp.starts_with(b"0x") {
        &imp[2..]
    } else {
        return Err("public key should be prefixed with '0x'");
    };

    // check key is correct len
    if imp.len() != 64 {
        return Err("256 bit public key should be 64 hex digits");
    }

    // decode hex
    let pk: Vec<u8> = hex::decode(imp).map_err(|err| {
        use hex::FromHexError::*;
        match err {
            InvalidHexCharacter { .. } => "invalid hex character, must be [0-9][a-z][A-Z]",
            OddLength => panic!("this should not happen"),
            InvalidStringLength => panic!("this should not happen"),
        }
    })?;

    assert_eq!(pk.len(), 32);

    Ok(slice_to_arr32(&pk))
}
