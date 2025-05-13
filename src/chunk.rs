use std::{fmt::Display, string::FromUtf8Error};

use crate::chunk_type::ChunkType;

#[derive(Debug)]
pub struct Chunk {
    pub chunk_crc: u32,
    pub chunk_type: ChunkType,
    pub chunk_data: Vec<u8>,
    pub length: u32,
}

impl Chunk {
    fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let mut chunk_type_data = vec![];

        chunk_type_data.extend(chunk_type.bytes());
        chunk_type_data.extend(data.clone());

        Chunk {
            chunk_crc: crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC).checksum(&chunk_type_data),
            chunk_type,
            length: data.len() as u32,
            chunk_data: data,
        }
    }

    fn length(&self) -> u32 {
        self.length
    }

    fn crc(&self) -> u32 {
        self.chunk_crc
    }

    fn chunk_type(&self) -> ChunkType {
        self.chunk_type.clone()
    }

    fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.chunk_data.to_vec())
    }
}

impl TryFrom<&Vec<u8>> for Chunk {
    type Error = &'static str;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let vec_clone = value.to_vec();

        let length: [u8; 4] = vec_clone
            .iter()
            .take(4)
            .copied()
            .collect::<Vec<u8>>()
            .try_into()
            .map_err(|_| "Length err")?;
        let chunk_type: [u8; 4] = vec_clone
            .iter()
            .skip(4)
            .take(4)
            .copied()
            .collect::<Vec<u8>>()
            .try_into()
            .map_err(|_| "Chunk type error")?;
        let message = vec_clone
            .iter()
            .skip(8)
            .take(value.len() - 12)
            .copied()
            .collect::<Vec<u8>>();
        let crc_data: [u8; 4] = vec_clone
            .iter()
            .skip(value.len() - 4)
            .take(4)
            .copied()
            .collect::<Vec<u8>>()
            .try_into()
            .map_err(|_| "Crc err")?;

        let mut chunk_type_data = vec![];

        chunk_type_data.extend(chunk_type);
        chunk_type_data.extend(&message);

        let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC).checksum(&chunk_type_data);

        if crc != u32::from_be_bytes(crc_data) {
            return Err("Crc Err");
        }

        Ok(Chunk {
            chunk_crc: crc,
            chunk_type: ChunkType(chunk_type),
            chunk_data: message,
            length: u32::from_be_bytes(length),
        })
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let crc = self.crc();
        let chunk_type = self.chunk_type.to_string();
        let chunk_data = String::from_utf8(self.chunk_data.to_vec()).expect("Chunk data err");

        write!(
            f,
            "CRC: {}, Chunk type: {}, Chunk data: {}, Length: {}",
            crc,
            chunk_type,
            chunk_data,
            self.length()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
