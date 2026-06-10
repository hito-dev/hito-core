/// c0m 
///
/// A minimal, zero-alloc, zero-copy command protocol for embedded/HWW use.
///
/// # Text frame (debug)
///
///   <command>\x20<hex_param_1>\x20...<hex_param_n>\n
///
/// # Binary frame
///
///   <command>\x1F<u8:param_count>(<u16_BE:len><bytes>)...
///
/// Absent (optional) parameters are encoded as zero-length: [0x00, 0x00]
use crate::drivers::transport::Transport;
use crate::drivers::payload::PayloadError;

/// Maximum number of parameters a command can carry.
pub const MAX_PARAMS: usize = 8; // TODO - remove this limit or make it configurable

/// Maximum length of a command string e.g. "zcash.sign"
pub const MAX_CMD_LEN: usize = 64;

/// Separator bytes
pub const SEP_TEXT: u8 = 0x20;   // space
pub const SEP_BINARY: u8 = 0x1F; // unit separator ␟
pub const TERM_TEXT: u8 = 0x0A;  // newline \n

// ─── Param ───────────────────────────────────────────────────────────────────

/// A single parameter — a slice into the original buffer (zero-copy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Param<'a> {
    /// Parameter is present with this data.
    Present(&'a [u8]),
    /// Parameter was explicitly absent (zero-length sentinel).
    Absent,
}

impl<'a> Param<'a> {
    pub fn is_present(&self) -> bool {
        matches!(self, Param::Present(_))
    }

    pub fn data(&self) -> Option<&'a [u8]> {
        match self {
            Param::Present(d) => Some(d),
            Param::Absent => None,
        }
    }
}

// ─── Command ─────────────────────────────────────────────────────────────────

/// A parsed c0m command. All slices point into the original input buffer.
//#[derive(Debug)]
pub struct Command<'a> {
    /// e.g. b"zcash.sign"
    pub name: &'a [u8],
    /// Parsed parameters (up to MAX_PARAMS).
    params: [Param<'a>; MAX_PARAMS],
    param_count: usize,
}

pub struct PendingCommand {
    pub consumed: usize,
    command_name: [u8; MAX_CMD_LEN],
    command_name_len: usize,
}

impl<'a> Command<'a> {
    /// Returns the number of parameters (including absent ones).
    pub fn param_count(&self) -> usize {
        self.param_count
    }

    /// Returns the parameter at `index`, or None if out of range.
    pub fn param(&self, index: usize) -> Option<Param<'a>> {
        if index < self.param_count {
            Some(self.params[index])
        } else {
            None
        }
    }

    /// Convenience: get present data at index or None.
    pub fn data(&self, index: usize) -> Option<&'a [u8]> {
        self.param(index)?.data()
    }

    /// Returns command name as str (if valid UTF-8).
    pub fn name_str(&self) -> Option<&str> {
        core::str::from_utf8(self.name).ok()
    }
}

impl PendingCommand {
    fn command_name(&self) -> &[u8] {
        &self.command_name[..self.command_name_len]
    }
    pub fn command_name_str(&self) -> Option<&str> {
        core::str::from_utf8(self.command_name()).ok()
    }

    pub fn command<'a>(&self, transport: &'a impl Transport) -> Result<Command<'a>, C0mTransportError> {
        // We only parse the command name in poll(), so we need to re-parse the full command here.
        // This is a bit redundant but keeps the parsing logic simple and zero-copy.
        let payload = transport.payload();
        let frame = &payload[..self.consumed];

        let (cmd, consumed) = parse(frame)?;
        if consumed != self.consumed {
            // This should never happen if the parser is consistent, but we check just in case.
            return Err(C0mTransportError::PayloadChanged);
        }

        Ok(cmd)
    }

    pub fn complete(&self, transport: &mut impl Transport) {
        transport.consume_payload(self.consumed);
    }
}

#[derive(Debug)]
pub enum C0mTransportError {
    PayloadChanged,
    PayloadError(PayloadError),
    Malformed(C0mError),
}
impl From<PayloadError> for C0mTransportError {
    fn from(err: PayloadError) -> Self {
        C0mTransportError::PayloadError(err)
    }
}
impl From<C0mError> for C0mTransportError {
    fn from(err: C0mError) -> Self {
        C0mTransportError::Malformed(err)
    }
}

pub fn poll<T: Transport>(
    transport: &mut T,
    //_now_ms: u64,
) -> Result<Option<PendingCommand>, C0mTransportError> {
    transport.poll_rx()?;

    if !transport.has_data() {
        return Ok(None);
    }

    let input = transport.payload();

    match parse(input) {
        Ok((cmd, consumed)) => {
            let mut command_name = [0u8; MAX_CMD_LEN];
            command_name[..cmd.name.len()].copy_from_slice(cmd.name);

            Ok(Some(PendingCommand {
                consumed,
                command_name,
                command_name_len: cmd.name.len(),
            }))
        }
        Err(C0mError::MissingTerminator) | Err(C0mError::TooShort) | Err(C0mError::Truncated) => {
            // Wait for more data
            Ok(None)
        }
        Err(err) => {
            // Parsing error — consume all data to avoid repeated errors
            transport.clear_payload();
            Err(C0mTransportError::Malformed(err))
        }
    }
}

// ─── Error ───────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum C0mError {
    /// Input buffer is empty or too short.
    TooShort,
    /// Command name exceeds MAX_CMD_LEN.
    CommandTooLong,
    /// No separator found (neither SP nor US).
    NoSeparator,
    /// param_count in binary frame exceeds MAX_PARAMS.
    TooManyParams,
    /// Binary frame is truncated (not enough bytes for declared length).
    Truncated,
    /// Text frame hex decoding failed.
    InvalidHex,
    /// Text frame missing newline terminator.
    MissingTerminator,
}

// ─── Parser ──────────────────────────────────────────────────────────────────

/// Parse a c0m frame from `input`.
///
/// Automatically detects text vs binary mode from the separator byte.
/// Returns a [`Command`] borrowing from `input` (zero-copy),
/// and the number of bytes consumed.
pub fn parse(input: &[u8]) -> Result<(Command<'_>, usize), C0mError> {
    if input.is_empty() {
        return Err(C0mError::TooShort);
    }

    // Allow zero-param text command:
    //
    //   ping\n
    //
    if let Some(pos) = input.iter().position(|&b| b == TERM_TEXT) {
        let before_term = &input[..pos];

        if !before_term.is_empty()
            && !before_term.contains(&SEP_TEXT)
            && !before_term.contains(&SEP_BINARY)
        {
            if before_term.len() > MAX_CMD_LEN {
                return Err(C0mError::CommandTooLong);
            }

            return Ok((
                Command {
                    name: before_term,
                    params: [Param::Absent; MAX_PARAMS],
                    param_count: 0,
                },
                pos + 1,
            ));
        }
    }


    // Find the first SP or US to locate end of command name
    let sep_pos = input
        .iter()
        .position(|&b| b == SEP_TEXT || b == SEP_BINARY)
        .ok_or(C0mError::NoSeparator)?;

    if sep_pos > MAX_CMD_LEN {
        return Err(C0mError::CommandTooLong);
    }

    let name = &input[..sep_pos];
    let sep = input[sep_pos];
    let rest = &input[sep_pos + 1..];

    match sep {
        SEP_BINARY => parse_binary(name, rest),
        SEP_TEXT => parse_text(name, rest),
        _ => unreachable!(),
    }
}

// ─── Binary parser ────────────────────────────────────────────────────────────

fn parse_binary<'a>(
    name: &'a [u8],
    rest: &'a [u8],
) -> Result<(Command<'a>, usize), C0mError> {
    // First byte: param count
    if rest.is_empty() {
        return Err(C0mError::TooShort);
    }
    let param_count = rest[0] as usize;
    if param_count > MAX_PARAMS {
        return Err(C0mError::TooManyParams);
    }

    let mut params = [Param::Absent; MAX_PARAMS];
    let mut cursor = 1usize; // offset into `rest` after param_count byte

    for i in 0..param_count {
        // Need 2 bytes for u16 length
        if cursor + 2 > rest.len() {
            return Err(C0mError::Truncated);
        }
        let len = u16::from_be_bytes([rest[cursor], rest[cursor + 1]]) as usize;
        cursor += 2;

        if len == 0 {
            params[i] = Param::Absent;
        } else {
            if cursor + len > rest.len() {
                return Err(C0mError::Truncated);
            }
            params[i] = Param::Present(&rest[cursor..cursor + len]);
            cursor += len;
        }
    }

    // Total consumed = name.len() + 1 (sep) + 1 (param_count) + cursor
    let consumed = name.len() + 1 + cursor;

    Ok((Command { name, params, param_count }, consumed))
}

// ─── Text parser ──────────────────────────────────────────────────────────────

fn parse_text<'a>(
    name: &'a [u8],
    rest: &'a [u8],
) -> Result<(Command<'a>, usize), C0mError> {
    // Find newline terminator
    let line_len = rest
        .iter()
        .position(|&b| b == TERM_TEXT)
        .ok_or(C0mError::MissingTerminator)?;

    let line = &rest[..line_len];

    let mut params = [Param::Absent; MAX_PARAMS];
    let mut param_count = 0;

    // Split on SP, decode each hex token
    for token in line.split(|&b| b == SEP_TEXT) {
        if param_count >= MAX_PARAMS {
            return Err(C0mError::TooManyParams);
        }
        if token.is_empty() {
            params[param_count] = Param::Absent;
        } else {
            // Validate hex — we keep a slice of the hex bytes (not decoded)
            // Caller decodes if needed; this keeps us zero-alloc.
            if !is_valid_hex(token) {
                return Err(C0mError::InvalidHex);
            }
            params[param_count] = Param::Present(token); // raw hex slice
        }
        param_count += 1;
    }

    let consumed = name.len() + 1 + line_len + 1; // +1 sep, +1 \n

    Ok((Command { name, params, param_count }, consumed))
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn is_valid_hex(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes.len() % 2 == 0
        && bytes.iter().all(|b| b.is_ascii_hexdigit())
}

/// Decode a hex slice into `out`. Returns number of bytes written.
/// `out` must be at least `hex.len() / 2` bytes long.
pub fn decode_hex(hex: &[u8], out: &mut [u8]) -> Result<usize, C0mError> {
    if hex.len() % 2 != 0 || out.len() < hex.len() / 2 {
        return Err(C0mError::InvalidHex);
    }
    for (i, chunk) in hex.chunks(2).enumerate() {
        out[i] = hex_byte(chunk[0])? << 4 | hex_byte(chunk[1])?;
    }
    Ok(hex.len() / 2)
}

fn hex_byte(b: u8) -> Result<u8, C0mError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(C0mError::InvalidHex),
    }
}

// ─── Frame builder ────────────────────────────────────────────────────────────

/// Write a binary c0m frame into `out`. Returns bytes written.
///
/// `params` — slice of optional byte slices; `None` = absent param.
pub fn encode_binary<'a>(
    cmd: &[u8],
    params: &[Option<&[u8]>],
    out: &mut [u8],
) -> Result<usize, C0mError> {
    let mut pos = 0;

    // command name
    let needed = cmd.len() + 1 + 1 + params.iter().map(|p| 2 + p.map_or(0, |d| d.len())).sum::<usize>();
    if out.len() < needed {
        return Err(C0mError::Truncated);
    }

    out[pos..pos + cmd.len()].copy_from_slice(cmd);
    pos += cmd.len();

    out[pos] = SEP_BINARY;
    pos += 1;

    out[pos] = params.len() as u8;
    pos += 1;

    for param in params {
        match param {
            None => {
                out[pos] = 0;
                out[pos + 1] = 0;
                pos += 2;
            }
            Some(data) => {
                let len = data.len() as u16;
                out[pos..pos + 2].copy_from_slice(&len.to_be_bytes());
                pos += 2;
                out[pos..pos + data.len()].copy_from_slice(data);
                pos += data.len();
            }
        }
    }

    Ok(pos)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_round_trip() {
        let pczt = b"\xde\xad\xbe\xef";
        let meta = b"\xca\xfe";

        let mut buf = [0u8; 128];
        let written = encode_binary(
            b"zcash.sign",
            &[Some(pczt), Some(meta)],
            &mut buf,
        ).unwrap();

        let (cmd, consumed) = parse(&buf[..written]).unwrap();

        assert_eq!(consumed, written);
        assert_eq!(cmd.name, b"zcash.sign");
        assert_eq!(cmd.param_count(), 2);
        assert_eq!(cmd.data(0), Some(pczt.as_ref()));
        assert_eq!(cmd.data(1), Some(meta.as_ref()));
    }

    #[test]
    fn test_binary_absent_param() {
        let pczt = b"\xde\xad\xbe\xef";

        let mut buf = [0u8; 128];
        let written = encode_binary(
            b"zcash.sign",
            &[Some(pczt), None],
            &mut buf,
        ).unwrap();

        let (cmd, _) = parse(&buf[..written]).unwrap();
        assert_eq!(cmd.param_count(), 2);
        assert_eq!(cmd.param(0), Some(Param::Present(pczt.as_ref())));
        assert_eq!(cmd.param(1), Some(Param::Absent));
    }

    #[test]
    fn test_text_frame() {
        let input = b"zcash.sign deadbeef cafebabe\n";
        let (cmd, consumed) = parse(input).unwrap();
        assert_eq!(consumed, input.len());
        assert_eq!(cmd.name, b"zcash.sign");
        assert_eq!(cmd.param_count(), 2);
        assert_eq!(cmd.data(0), Some(b"deadbeef".as_ref()));
        assert_eq!(cmd.data(1), Some(b"cafebabe".as_ref()));
    }

    #[test]
    fn test_text_no_newline() {

        let input = b"zcash.sign deadbeef";
        //assert_eq!(parse(input), Err(C0mError::MissingTerminator));
        assert!(matches!(parse(input), Err(C0mError::MissingTerminator)));
    }

    #[test]
    fn test_binary_truncated() {
        // encode then truncate
        let mut buf = [0u8; 128];
        let written = encode_binary(b"zcash.sign", &[Some(b"\xde\xad")], &mut buf).unwrap();
        assert!(parse(&buf[..written - 1]).is_err());
    }

    #[test]
    fn test_text_zero_param_command() {
        let input = b"ping\n";

        let (cmd, consumed) = parse(input).unwrap();

        assert_eq!(consumed, input.len());
        assert_eq!(cmd.name, b"ping");
        assert_eq!(cmd.param_count(), 0);
    }

}
