#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Opcode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatusRequest = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
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
