use thrift::protocol::{TBinaryInputProtocol, TBinaryOutputProtocol, TSerializable};
use thrift::transport::TBufferChannel;

pub fn thrift_serialize<T: TSerializable>(data: &T) -> anyhow::Result<Vec<u8>> {
    let mut transport = TBufferChannel::with_capacity(0, 1024);
    let mut o_prot = TBinaryOutputProtocol::new(&mut transport, true);
    data.write_to_out_protocol(&mut o_prot)?;
    Ok(transport.write_bytes().to_vec())
}

pub fn thrift_deserialize<T: TSerializable>(bytes: &[u8]) -> anyhow::Result<T> {
    let mut transport = TBufferChannel::with_capacity(bytes.len(), 0);
    transport.set_readable_bytes(bytes);
    let mut i_prot = TBinaryInputProtocol::new(&mut transport, true);
    T::read_from_in_protocol(&mut i_prot).map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntUser;
    use chrono::Utc;

    #[test]
    fn test_thrift_user_serialization() {
        let original = EntUser {
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            full_name: Some("Test User".to_string()),
            bio: Some("A test user".to_string()),
            profile_picture_url: None,
            created_time: Utc::now().timestamp(),
            updated_time: None,
            last_active_time: Some(Utc::now().timestamp()),
            is_verified: false,
            location: Some("Test City".to_string()),
        };

        let serialized = thrift_serialize(&original).unwrap();
        let deserialized: EntUser = thrift_deserialize(&serialized).unwrap();

        assert_eq!(original.username, deserialized.username);
        assert_eq!(original.email, deserialized.email);
        assert_eq!(original.full_name, deserialized.full_name);
        assert_eq!(original.bio, deserialized.bio);
        assert_eq!(original.is_verified, deserialized.is_verified);
    }
}