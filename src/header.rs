use crate::error::{DnsError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Opcode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatusRequest = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsHeader {
    /// A random ID assigned to query packets. Response packets must reply with the same ID.
    pub id: u16,
    /// 1 for a reply packet, 0 for a question packet.
    pub qr_indicator: bool,
    /// Specifies the kind of query in a message.
    pub opcode: Opcode,
    /// 1 if the responding server "owns" the domain queried, i.e., it's authoritative.
    pub authoritative_answer: bool,
    /// 1 if the message is larger than 512 bytes. Always 0 in UDP responses.
    pub truncation: bool,
    /// Sender sets this to 1 if the server should recursively resolve this query, 0 otherwise.
    pub recursion_desired: bool,
    /// Server sets this to 1 to indicate that recursion is available.
    pub recursion_available: bool,
    /// Used by DNSSEC queries. At inception, it was reserved for future use.
    pub reserved: u8,
    /// Response code indicating the status of the response.
    pub response_code: u8,
    /// Number of questions in the Question section.
    pub question_count: u16,
    /// Number of records in the Answer section.
    pub answer_record_count: u16,
    /// Number of records in the Authority section.
    pub authority_record_count: u16,
    /// Number of records in the Additional section.
    pub additional_record_count: u16,
}

impl DnsHeader {
    pub fn try_parse(buf: &[u8]) -> Result<Self> {
        if buf.len() < 12 {
            return Err(DnsError::Parse);
        }

        let id = u16::from_be_bytes([buf[0], buf[1]]);
        let qr_indicator = buf[2] & 0b1000_0000 != 0;
        let opcode = match buf[2] & 0b0111_1000 {
            0b0000_0000 => Opcode::StandardQuery,
            0b0000_1000 => Opcode::InverseQuery,
            0b0001_0000 => Opcode::ServerStatusRequest,
            _ => return Err(DnsError::Parse),
        };
        let authoritative_answer = buf[2] & 0b0000_0100 != 0;
        let truncation = buf[2] & 0b0000_0010 != 0;
        let recursion_desired = buf[2] & 0b0000_0001 != 0;
        let recursion_available = buf[3] & 0b1000_0000 != 0;
        let reserved = (buf[3] & 0b0111_0000) >> 4;
        let response_code = buf[3] & 0b0000_1111;
        let question_count = u16::from_be_bytes([buf[4], buf[5]]);
        let answer_record_count = u16::from_be_bytes([buf[6], buf[7]]);
        let authority_record_count = u16::from_be_bytes([buf[8], buf[9]]);
        let additional_record_count = u16::from_be_bytes([buf[10], buf[11]]);

        Ok(Self {
            id,
            qr_indicator,
            opcode,
            authoritative_answer,
            truncation,
            recursion_desired,
            recursion_available,
            reserved,
            response_code,
            question_count,
            answer_record_count,
            authority_record_count,
            additional_record_count,
        })
    }

    pub fn serialize(&self) -> [u8; 12] {
        [
            self.id.to_be_bytes()[0],
            self.id.to_be_bytes()[1],
            ((self.qr_indicator as u8) << 7)
                | ((self.opcode as u8) << 3)
                | ((self.authoritative_answer as u8) << 2)
                | ((self.truncation as u8) << 1)
                | (self.recursion_desired as u8),
            ((self.recursion_available as u8) << 7) | (self.reserved << 4) | self.response_code,
            self.question_count.to_be_bytes()[0],
            self.question_count.to_be_bytes()[1],
            self.answer_record_count.to_be_bytes()[0],
            self.answer_record_count.to_be_bytes()[1],
            self.authority_record_count.to_be_bytes()[0],
            self.authority_record_count.to_be_bytes()[1],
            self.additional_record_count.to_be_bytes()[0],
            self.additional_record_count.to_be_bytes()[1],
        ]
    }
}
