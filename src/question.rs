use std::io::Write;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QType {
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

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QClass {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsQuestion {
    pub name: Vec<Label>,
    pub qtype: QType,
    pub class: QClass,
}

impl Label {
    pub fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        buf.write_all(&[self.content.len() as u8])?;
        buf.write_all(self.content.as_bytes())
    }
}

impl DnsQuestion {
    pub fn serialize<W: Write>(&self, buf: &mut W) -> std::io::Result<()> {
        for label in &self.name {
            label.serialize(buf)?;
        }

        let qtype = (self.qtype as u16).to_be_bytes();
        let class = (self.class as u16).to_be_bytes();
        buf.write_all(&[0, qtype[0], qtype[1], class[0], class[1]])
    }
}
