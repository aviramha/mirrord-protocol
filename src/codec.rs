use actix_codec::{Decoder, Encoder};
use bincode::{error::DecodeError, Decode, Encode};
use bytes::{Buf, BufMut, BytesMut};
use std::io;

type ConnectionID = u16;

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct NewTCPConnection {
    pub connection_id: ConnectionID,
    pub port: u16,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct TCPData {
    pub connection_id: ConnectionID,
    pub data: Vec<u8>,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct TCPClose {
    pub connection_id: ConnectionID,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct LogMessage {
    pub message: String,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub enum MirrordMessage {
    Close,
    NewTCPConnection(NewTCPConnection),
    TCPData(TCPData),
    TCPClose(TCPClose),
    LogMessage(LogMessage),
}

pub struct MirrordCodec {
    config: bincode::config::Configuration,
}

impl MirrordCodec {
    pub fn new() -> Self {
        MirrordCodec {
            config: bincode::config::standard(),
        }
    }
}

impl Default for MirrordCodec {
    fn default() -> Self {
        MirrordCodec::new()
    }
}

impl Decoder for MirrordCodec {
    type Item = MirrordMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        match bincode::decode_from_slice(&src[..], self.config) {
            Ok((message, read)) => {
                src.advance(read);
                Ok(Some(message))
            }
            Err(DecodeError::UnexpectedEnd) => Ok(None),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
        }
    }
}

impl Encoder<MirrordMessage> for MirrordCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: MirrordMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = match bincode::encode_to_vec(msg, self.config) {
            Ok(encoded) => encoded,
            Err(err) => {
                return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
            }
        };
        dst.reserve(encoded.len());
        dst.put(&encoded[..]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    #[test]
    fn sanity_encode_decode() {
        let mut codec = MirrordCodec::new();
        let mut buf = BytesMut::new();

        let msg = MirrordMessage::NewTCPConnection(NewTCPConnection {
            connection_id: 1,
            port: 8080,
        });

        codec.encode(msg.clone(), &mut buf).unwrap();

        let decoded = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded, msg);
        assert!(buf.is_empty());
    }

    #[test]
    fn decode_invalid_data() {
        let mut codec = MirrordCodec::new();
        let mut buf = BytesMut::new();
        buf.put_u8(254);

        let res = codec.decode(&mut buf);
        match res {
            Ok(_) => panic!("Should have failed"),
            Err(err) => assert_eq!(err.kind(), io::ErrorKind::Other),
        }
    }

    #[test]
    fn decode_partial_data() {
        let mut codec = MirrordCodec::new();
        let mut buf = BytesMut::new();
        buf.put_u8(1);

        assert!(codec.decode(&mut buf).unwrap().is_none());
    }
}