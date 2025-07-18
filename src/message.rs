use std::{
    borrow::Cow,
    io::{Cursor, Write},
};

use bytes::{Buf, TryGetError};

use crate::error::{DnsError, Result};

/// Trait defining behavior for types that can be serialized into bytes.
///
/// Implementors of this trait can write their data to any type that
/// implements the [Write] trait.
pub trait ByteSerialize {
    /// Serializes the implementing type into bytes and writes them to the provided buffer.
    ///
    /// # Arguments
    /// * `buf` - A mutable reference to something that implements [Write]
    ///
    /// # Returns
    /// * Success or an I/O error
    fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Opcode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatusRequest = 2,
    Invalid,
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
    pub fn try_parse(buf: &mut Cursor<&[u8]>) -> Result<Self> {
        if buf.remaining() < 12 {
            return Err(DnsError::NotEnoughData(TryGetError {
                requested: 12,
                available: buf.remaining(),
            }));
        }

        let id = buf.get_u16();

        let b = buf.get_u8();
        let qr_indicator = b & 0b1000_0000 != 0;
        let opcode = match b & 0b0111_1000 {
            0b0000_0000 => Opcode::StandardQuery,
            0b0000_1000 => Opcode::InverseQuery,
            0b0001_0000 => Opcode::ServerStatusRequest,
            _ => Opcode::Invalid,
        };
        let authoritative_answer = b & 0b0000_0100 != 0;
        let truncation = b & 0b0000_0010 != 0;
        let recursion_desired = b & 0b0000_0001 != 0;

        let b = buf.get_u8();
        let recursion_available = b & 0b1000_0000 != 0;
        let reserved = (b & 0b0111_0000) >> 4;
        let response_code = b & 0b0000_1111;

        let question_count = buf.get_u16();
        let answer_record_count = buf.get_u16();
        let authority_record_count = buf.get_u16();
        let additional_record_count = buf.get_u16();

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
}

impl ByteSerialize for DnsHeader {
    fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        buf.write_all(&[
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
        ])
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /// a host address
    A = 1,
    /// an authoritative name server
    NS = 2,
    /// a mail destination (Obsolete - use MX)
    MD = 3,
    /// a mail forwarder (Obsolete - use MX)
    MF = 4,
    /// the canonical name for an alias
    CNAME = 5,
    /// marks the start of a zone of authority
    SOA = 6,
    /// a mailbox domain name (EXPERIMENTAL)
    MB = 7,
    /// a mail group member (EXPERIMENTAL)
    MG = 8,
    /// a mail rename domain name (EXPERIMENTAL)
    MR = 9,
    /// a null RR (EXPERIMENTAL)
    NULL = 10,
    /// a well known service description
    WKS = 11,
    /// a domain name pointer
    PTR = 12,
    /// host information
    HINFO = 13,
    /// mailbox or mail list information
    MINFO = 14,
    /// mail exchange
    MX = 15,
    /// text strings
    TXT = 16,

    // Question Type Only
    /// A request for a transfer of an entire zone
    AXFR = 252,
    /// A request for mailbox-related records (MB, MG or MR)
    MAILB = 253,
    /// A request for mail agent RRs (Obsolete - see MX)
    MAILA = 254,
    /// A request for all records
    Wildcard = 255,
}

impl TryFrom<u16> for Type {
    type Error = DnsError;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Type::A),
            2 => Ok(Type::NS),
            3 => Ok(Type::MD),
            4 => Ok(Type::MF),
            5 => Ok(Type::CNAME),
            6 => Ok(Type::SOA),
            7 => Ok(Type::MB),
            8 => Ok(Type::MG),
            9 => Ok(Type::MR),
            10 => Ok(Type::NULL),
            11 => Ok(Type::WKS),
            12 => Ok(Type::PTR),
            13 => Ok(Type::HINFO),
            14 => Ok(Type::MINFO),
            15 => Ok(Type::MX),
            16 => Ok(Type::TXT),

            252 => Ok(Type::AXFR),
            253 => Ok(Type::MAILB),
            254 => Ok(Type::MAILA),
            255 => Ok(Type::Wildcard),

            _ => Err(DnsError::InvalidType),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// The Internet
    IN = 1,
    /// The CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CS = 2,
    /// The CHAOS class
    CH = 3,
    /// Hesiod [Dyer 87]
    HS = 4,

    // Question Class Only
    /// Any class
    Wildcard = 255,
}

impl TryFrom<u16> for Class {
    type Error = DnsError;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Class::IN),
            2 => Ok(Class::CS),
            3 => Ok(Class::CH),
            4 => Ok(Class::HS),

            255 => Ok(Class::Wildcard),

            _ => Err(DnsError::InvalidClass),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label<'packet> {
    section: Cow<'packet, [u8]>,
}

impl ByteSerialize for Label<'_> {
    fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        buf.write_all(&self.section)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name<'packet> {
    pub labels: Vec<Label<'packet>>,
}

impl<'packet> Name<'packet> {
    pub fn try_parse(buf: &mut Cursor<&'packet [u8]>) -> Result<Self> {
        let mut labels = Vec::new();
        loop {
            match buf.try_get_u8()? {
                0 => break,

                // compressed label
                p if p & 0b1100_0000 == 0b1100_0000 => {
                    let b1 = p & 0b0011_1111;
                    let b2 = buf.try_get_u8()?;
                    let offset = u16::from_be_bytes([b1, b2]) as usize;

                    // check offset bounds
                    if offset >= buf.get_ref().len() {
                        return Err(DnsError::InvalidName("reference out of bounds"));
                    }

                    let packet = buf.get_ref();
                    let pos = packet[offset..]
                        .iter()
                        .position(|&b| b == 0)
                        .ok_or(DnsError::InvalidName("null terminator missing"))?;

                    labels.push(Label {
                        section: Cow::from(&packet[offset..offset + pos]),
                    });

                    break;
                }

                // uncompressed label
                len => {
                    if buf.remaining() < len as usize {
                        return Err(DnsError::NotEnoughData(TryGetError {
                            requested: len as usize,
                            available: buf.remaining(),
                        }));
                    }
                    let pos = buf.position() as usize;
                    let len = len as usize;
                    let section = &buf.get_ref()[pos - 1..pos + len];
                    buf.advance(len);

                    labels.push(Label {
                        section: Cow::from(section),
                    });
                }
            }

            if labels.len() > 30 {
                // Constrain the maximum number of labels allowed
                break;
            }
        }

        Ok(Self { labels })
    }
}

impl ByteSerialize for Name<'_> {
    fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        for label in &self.labels {
            label.serialize(buf)?;
        }
        buf.write_all(&[0])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsQuestion<'packet> {
    pub name: Name<'packet>,
    pub qtype: Type,
    pub class: Class,
}

impl<'packet> DnsQuestion<'packet> {
    pub fn try_parse(buf: &mut Cursor<&'packet [u8]>) -> Result<Self> {
        let name = Name::try_parse(buf)?;
        let qtype = Type::try_from(buf.try_get_u16()?)?;
        let class = Class::try_from(buf.try_get_u16()?)?;

        Ok(Self { name, qtype, class })
    }
}

impl ByteSerialize for DnsQuestion<'_> {
    fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        self.name.serialize(buf)?;

        let qtype = (self.qtype as u16).to_be_bytes();
        let class = (self.class as u16).to_be_bytes();
        buf.write_all(&[qtype[0], qtype[1], class[0], class[1]])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RData {
    A { address: u32 },
}

impl ByteSerialize for RData {
    fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        match *self {
            RData::A { address } => {
                let address = address.to_be_bytes();
                buf.write_all(&[0, 4, address[0], address[1], address[2], address[3]])
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRecord<'packet> {
    pub name: Name<'packet>,
    pub atype: Type,
    pub class: Class,
    pub ttl: u32,
    pub rdata: RData,
}

impl<'packet> ResourceRecord<'packet> {
    pub fn try_parse(buf: &mut Cursor<&'packet [u8]>) -> Result<Self> {
        let name = Name::try_parse(buf)?;

        let atype = Type::try_from(buf.try_get_u16()?)?;
        let class = Class::try_from(buf.try_get_u16()?)?;
        let ttl = buf.try_get_u32()?;

        // This assumes atype is a Type::A
        let _rdlength = buf.try_get_u16()?;
        let address = buf.try_get_u32()?;

        Ok(Self {
            name,
            atype,
            class,
            ttl,
            rdata: RData::A { address },
        })
    }
}

impl ByteSerialize for ResourceRecord<'_> {
    fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        self.name.serialize(buf)?;

        let atype = (self.atype as u16).to_be_bytes();
        let class = (self.class as u16).to_be_bytes();
        let ttl = self.ttl.to_be_bytes();
        buf.write_all(&[
            atype[0], atype[1], class[0], class[1], ttl[0], ttl[1], ttl[2], ttl[3],
        ])?;

        self.rdata.serialize(buf)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsMessage<'packet> {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion<'packet>>,
    pub records: Vec<ResourceRecord<'packet>>,
}

impl<'packet> DnsMessage<'packet> {
    pub fn try_parse(buf: &mut Cursor<&'packet [u8]>) -> Result<Self> {
        let header = DnsHeader::try_parse(buf)?;

        let mut questions = Vec::new();
        for _ in 0..header.question_count {
            let question = DnsQuestion::try_parse(buf)?;
            questions.push(question);
        }

        let mut records = Vec::new();
        for _ in 0..header.answer_record_count {
            let record = ResourceRecord::try_parse(buf)?;
            records.push(record);
        }

        Ok(Self {
            header,
            questions,
            records,
        })
    }
}

impl ByteSerialize for DnsMessage<'_> {
    fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        self.header.serialize(buf)?;
        for question in &self.questions {
            question.serialize(buf)?;
        }
        for record in &self.records {
            record.serialize(buf)?;
        }

        Ok(())
    }
}
