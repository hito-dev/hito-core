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
use crate::drivers::payload;

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
    pub fn param(&self, index: usize) -> Result<&'a [u8], Error> {
        if index < self.param_count {
            match self.params[index] {
                Param::Present(data) => Ok(data),
                Param::Absent => Err(Error::MissingParameter),
            }
        } else {
            return Err(Error::MissingParameter);
        }
    }

    /// Convenience: get present data at index or None.
    pub fn data(&self, index: usize) -> Option<&'a [u8]> {
        match self.param(index) {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }

    /// Returns command name as str (if valid UTF-8).
    pub fn name_str(&self) -> Option<&str> {
        core::str::from_utf8(self.name).ok()
    }
}

impl PendingCommand {
    pub fn text_hex_to_binary_in_place(
        &self,
        transport: &mut impl Transport,
    ) -> Result<(), Error> {
        let payload = transport.payload_mut();

        let _new_consumed = text_hex_to_binary_in_place(
            &mut payload[..self.consumed],
        )?;

        // Do not update self.consumed.
        // The converted binary frame is shorter, but complete() must consume
        // the original text frame length so queued frames stay aligned.

        Ok(())
    }
}

impl PendingCommand {
    fn command_name(&self) -> &[u8] {
        &self.command_name[..self.command_name_len]
    }
    pub fn command_name_str(&self) -> Option<&str> {
        core::str::from_utf8(self.command_name()).ok()
    }

    pub fn command<'a>(&self, transport: &'a impl Transport) -> Result<Command<'a>, Error> {
        // We only parse the command name in poll(), so we need to re-parse the full command here.
        // This is a bit redundant but keeps the parsing logic simple and zero-copy.
        let payload = transport.payload();
        let frame = &payload[..self.consumed];

        let (cmd, consumed) = parse(frame)?;

        if consumed != self.consumed {
            // This is allowed only after text_hex_to_binary_in_place(),
            // because the rewritten binary frame is shorter than the original text frame.
            let is_binary = frame.get(cmd.name.len()).copied() == Some(SEP_BINARY);

            if !is_binary || consumed > self.consumed {
                return Err(Error::from(payload::Error::PayloadChanged));
            }
        }

        Ok(cmd)
    }

    pub fn complete(&self, transport: &mut impl Transport) {
        transport.consume_payload(self.consumed);
    }
}

pub fn poll<T: Transport>(
    transport: &mut T,
    //_now_ms: u64,
) -> Result<Option<PendingCommand>, Error> {
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
        Err(Error::MissingTerminator) | Err(Error::TooShort) | Err(Error::Truncated) | Err(Error::NoSeparator) => {
            // Wait for more data
            Ok(None)
        }
        Err(err) => {
            // Parsing error — consume all data to avoid repeated errors
            transport.clear_payload();
            Err(err)
        }
    }
}

// ─── Error ───────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
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
    // Param index out of range.
    MissingParameter,
    // Unknown command 
    UnknownCommand,
    Payload(payload::Error),
}
impl From<payload::Error> for Error {
    fn from(err: payload::Error) -> Self {
        Error::Payload(err)
    }
}

// ─── Parser ──────────────────────────────────────────────────────────────────

/// Parse a c0m frame from `input`.
///
/// Automatically detects text vs binary mode from the separator byte.
/// Returns a [`Command`] borrowing from `input` (zero-copy),
/// and the number of bytes consumed.
pub fn parse(input: &[u8]) -> Result<(Command<'_>, usize), Error> {
    if input.is_empty() {
        return Err(Error::TooShort);
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
                return Err(Error::CommandTooLong);
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
        .ok_or(Error::NoSeparator)?;

    if sep_pos > MAX_CMD_LEN {
        return Err(Error::CommandTooLong);
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
) -> Result<(Command<'a>, usize), Error> {
    // First byte: param count
    if rest.is_empty() {
        return Err(Error::TooShort);
    }
    let param_count = rest[0] as usize;
    if param_count > MAX_PARAMS {
        return Err(Error::TooManyParams);
    }

    let mut params = [Param::Absent; MAX_PARAMS];
    let mut cursor = 1usize; // offset into `rest` after param_count byte

    for i in 0..param_count {
        // Need 2 bytes for u16 length
        if cursor + 2 > rest.len() {
            return Err(Error::Truncated);
        }
        let len = u16::from_be_bytes([rest[cursor], rest[cursor + 1]]) as usize;
        cursor += 2;

        if len == 0 {
            params[i] = Param::Absent;
        } else {
            if cursor + len > rest.len() {
                return Err(Error::Truncated);
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
) -> Result<(Command<'a>, usize), Error> {
    // Find newline terminator
    let line_len = rest
        .iter()
        .position(|&b| b == TERM_TEXT)
        .ok_or(Error::MissingTerminator)?;

    let line = &rest[..line_len];

    let mut params = [Param::Absent; MAX_PARAMS];
    let mut param_count = 0;

    // Split on SP, decode each hex token
    for token in line.split(|&b| b == SEP_TEXT) {
        if param_count >= MAX_PARAMS {
            return Err(Error::TooManyParams);
        }
        if token.is_empty() {
            params[param_count] = Param::Absent;
        } else {
            // Validate hex — we keep a slice of the hex bytes (not decoded)
            // Caller decodes if needed; this keeps us zero-alloc.
            //if !is_valid_hex(token) {
                //return Err(Error::InvalidHex);
            //}
            params[param_count] = Param::Present(token); // raw hex slice
        }
        param_count += 1;
    }

    let consumed = name.len() + 1 + line_len + 1; // +1 sep, +1 \n

    Ok((Command { name, params, param_count }, consumed))
}

pub fn text_hex_to_binary_in_place(input: &mut [u8]) -> Result<usize, Error> {
    if input.is_empty() {
        return Err(Error::TooShort);
    }

    // Already binary. Nothing to convert.
    if input.iter().any(|&b| b == SEP_BINARY) {
        let (_, consumed) = parse(input)?;
        return Ok(consumed);
    }

    let term_pos = input
        .iter()
        .position(|&b| b == TERM_TEXT)
        .ok_or(Error::MissingTerminator)?;

    let frame_len = term_pos + 1;
    let frame = &mut input[..frame_len];

    let sep_pos = frame
        .iter()
        .position(|&b| b == SEP_TEXT)
        .ok_or(Error::NoSeparator)?;

    if sep_pos > MAX_CMD_LEN {
        return Err(Error::CommandTooLong);
    }

    let name_len = sep_pos;
    let params_start = sep_pos + 1;
    let params_end = term_pos;

    let params_area = &frame[params_start..params_end];

    let param_count = count_text_params(params_area)?;
    if param_count > MAX_PARAMS {
        return Err(Error::TooManyParams);
    }

    // New binary frame layout:
    //
    //   <command>\x1F<u8:param_count>(<u16_BE:len><bytes>)...
    //
    // We write from left to right. This is safe because every text hex param
    // must be encoded as:
    //
    //   0x + 2 hex chars per byte
    //
    // so each param has at least enough room for:
    //
    //   u16 len + decoded bytes
    //
    let mut read = params_start;
    let mut write = name_len;

    frame[write] = SEP_BINARY;
    write += 1;

    let param_count_pos = write;
    write += 1;

    for p in 0..param_count {
        // trace text
        trace!("Processing param {} text: {:?}", p, core::str::from_utf8(&frame[read..params_end]).unwrap_or("<invalid utf-8>"));
        let token_start = read;

        while read < params_end && frame[read] != SEP_TEXT {
            read += 1;
        }

        let token_end = read;

        if token_start == token_end {
            // Empty text token -> absent param.
            frame[write] = 0;
            frame[write + 1] = 0;
            write += 2;
        } else {
            if token_end - token_start < 2 {
                trace!("Token too short to be valid hex: {:?}", &frame[token_start..token_end]);
                return Err(Error::InvalidHex);
            }

            if frame[token_start] != b'0' || frame[token_start + 1] != b'x' {
                trace!("Token does not start with 0x: {:?}", &frame[token_start..token_end]);
                return Err(Error::InvalidHex);
            }

            let hex_start = token_start + 2;
            let hex_len = token_end - hex_start;

            if hex_len == 0 || hex_len % 2 != 0 {
                trace!("Hex part must have even length: {:?}", &frame[hex_start..token_end]);
                return Err(Error::InvalidHex);
            }

            let decoded_len = hex_len / 2;

            if decoded_len > u16::MAX as usize {
                return Err(Error::Truncated);
            }

            let decoded_len_write = write;
            write += 2;

            for i in 0..decoded_len {
                trace!("Decoding byte {} of param {}: hex {:?}{:?}, hex_start: {}", i, p, frame[hex_start + i * 2], frame[hex_start + i * 2 + 1], hex_start);
                let hi = hex_byte(frame[hex_start + i * 2])?;
                let lo = hex_byte(frame[hex_start + i * 2 + 1])?;
                frame[write + i] = (hi << 4) | lo;
            }

            frame[decoded_len_write..decoded_len_write + 2].copy_from_slice(&(decoded_len as u16).to_be_bytes());

            write += decoded_len;
        }

        // Skip separator.
        if read < params_end && frame[read] == SEP_TEXT {
            read += 1;
        }
    }

    // Write param count after processing all params
    frame[param_count_pos] = param_count as u8;

    Ok(write)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn is_valid_hex(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes.len() % 2 == 0
        && bytes.iter().all(|b| b.is_ascii_hexdigit())
}

/// Decode a hex slice into `out`. Returns number of bytes written.
/// `out` must be at least `hex.len() / 2` bytes long.
pub fn decode_hex(hex: &[u8], out: &mut [u8]) -> Result<usize, Error> {
    if hex.len() % 2 != 0 || out.len() < hex.len() / 2 {
        return Err(Error::InvalidHex);
    }
    for (i, chunk) in hex.chunks(2).enumerate() {
        out[i] = hex_byte(chunk[0])? << 4 | hex_byte(chunk[1])?;
    }
    Ok(hex.len() / 2)
}

fn hex_byte(b: u8) -> Result<u8, Error> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(Error::InvalidHex),
    }
}

fn count_text_params(params_area: &[u8]) -> Result<usize, Error> {
    if params_area.is_empty() {
        return Ok(0);
    }

    let mut count = 1usize;

    for &b in params_area {
        if b == SEP_TEXT {
            count += 1;

            if count > MAX_PARAMS {
                return Err(Error::TooManyParams);
            }
        }
    }

    Ok(count)
}

// ─── Frame builder ────────────────────────────────────────────────────────────

/// Write a binary c0m frame into `out`. Returns bytes written.
///
/// `params` — slice of optional byte slices; `None` = absent param.
pub fn encode_binary<'a>(
    cmd: &[u8],
    params: &[Option<&[u8]>],
    out: &mut [u8],
) -> Result<usize, Error> {
    let mut pos = 0;

    // command name
    let needed = cmd.len() + 1 + 1 + params.iter().map(|p| 2 + p.map_or(0, |d| d.len())).sum::<usize>();
    if out.len() < needed {
        return Err(Error::Truncated);
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
        assert_eq!(cmd.data(0), Some(pczt.as_ref()));
        assert_eq!(cmd.data(1), None);
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
        //assert_eq!(parse(input), Err(Error::MissingTerminator));
        assert!(matches!(parse(input), Err(Error::MissingTerminator)));
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

    #[test]
    fn test_text_hex_to_binary_in_place() {
        let mut input = *b"ping 0x0f 0xf0\n";

        let consumed = text_hex_to_binary_in_place(&mut input).unwrap();

        let expected = b"ping\x1F\x02\x00\x01\x0f\x00\x01\xf0";

        assert_eq!(consumed, expected.len());
        assert_eq!(&input[..consumed], expected);

        let (cmd, parsed_consumed) = parse(&input[..consumed]).unwrap();

        assert_eq!(parsed_consumed, consumed);
        assert_eq!(cmd.name, b"ping");
        assert_eq!(cmd.param_count(), 2);
        assert_eq!(cmd.data(0), Some([0x0f].as_ref()));
        assert_eq!(cmd.data(1), Some([0xf0].as_ref()));
    }

    #[test]
    fn test_pending_text_hex_to_binary_in_place_then_reparse() {
        struct TestTransport {
            payload: [u8; 128],
            len: usize,
        }

        impl TestTransport {
            fn new(input: &[u8]) -> Self {
                let mut payload = [0u8; 128];
                payload[..input.len()].copy_from_slice(input);

                Self {
                    payload,
                    len: input.len(),
                }
            }
        }

        impl Transport for TestTransport {
            fn init(&mut self) -> bool { true }
            fn send(&mut self, _data: &[u8]) -> bool { true }
            fn poll_rx(&mut self) -> Result<(), payload::Error> {
                Ok(())
            }

            fn has_data(&self) -> bool {
                self.len > 0
            }

            fn payload(&self) -> &[u8] {
                &self.payload[..self.len]
            }

            fn payload_mut(&mut self) -> &mut [u8] {
                &mut self.payload[..self.len]
            }

            fn consume_payload(&mut self, consumed: usize) {
                self.payload.copy_within(consumed..self.len, 0);
                self.len -= consumed;
            }

            fn clear_payload(&mut self) {
                self.len = 0;
            }
        }

        let mut transport = TestTransport::new(b"ping 0x0f 0xf0\n");

        let pending = poll(&mut transport)
            .unwrap()
            .expect("pending command");

        assert_eq!(pending.command_name_str(), Some("ping"));

        pending
            .text_hex_to_binary_in_place(&mut transport)
            .unwrap();

        let cmd = pending.command(&transport).unwrap();

        assert_eq!(cmd.name, b"ping");
        assert_eq!(cmd.param_count(), 2);
        assert_eq!(cmd.data(0), Some([0x0f].as_ref()));
        assert_eq!(cmd.data(1), Some([0xf0].as_ref()));

        // The original text frame should now have been rewritten into binary layout.
        assert_eq!(
            transport.payload().get(..pending.consumed).unwrap(),
            b"ping\x1F\x02\x00\x01\x0f\x00\x01\xf0f0\n"
        );
    }

    #[test]
    fn test_pending_text_hex_to_binary_keeps_next_command_aligned() {
        struct TestTransport {
            payload: [u8; 128],
            len: usize,
        }

        impl TestTransport {
            fn new(input: &[u8]) -> Self {
                let mut payload = [0u8; 128];
                payload[..input.len()].copy_from_slice(input);

                Self {
                    payload,
                    len: input.len(),
                }
            }
        }

        impl Transport for TestTransport {
            fn init(&mut self) -> bool {
                true
            }

            fn send(&mut self, _data: &[u8]) -> bool {
                true
            }

            fn poll_rx(&mut self) -> Result<(), payload::Error> {
                Ok(())
            }

            fn has_data(&self) -> bool {
                self.len > 0
            }

            fn payload(&self) -> &[u8] {
                &self.payload[..self.len]
            }

            fn payload_mut(&mut self) -> &mut [u8] {
                &mut self.payload[..self.len]
            }

            fn consume_payload(&mut self, consumed: usize) {
                self.payload.copy_within(consumed..self.len, 0);
                self.len -= consumed;
            }

            fn clear_payload(&mut self) {
                self.len = 0;
            }
        }

        let input = b"ping 0x0f 0xf0\npong with_txt\n";
        let first_frame_len = b"ping 0x0f 0xf0\n".len();

        let mut transport = TestTransport::new(input);

        let pending = poll(&mut transport)
            .unwrap()
            .expect("first pending command");

        assert_eq!(pending.command_name_str(), Some("ping"));
        assert_eq!(pending.consumed, first_frame_len);

        pending
            .text_hex_to_binary_in_place(&mut transport)
            .unwrap();

        // consumed must remain the original text-frame length.
        assert_eq!(pending.consumed, first_frame_len);

        let cmd = pending.command(&transport).unwrap();

        assert_eq!(cmd.name, b"ping");
        assert_eq!(cmd.param_count(), 2);
        assert_eq!(cmd.data(0), Some([0x0f].as_ref()));
        assert_eq!(cmd.data(1), Some([0xf0].as_ref()));

        // The rewritten binary frame exists at the start of the original frame.
        let expected_binary_prefix = b"ping\x1F\x02\x00\x01\x0f\x00\x01\xf0";
        assert_eq!(
            &transport.payload()[..expected_binary_prefix.len()],
            expected_binary_prefix
        );

        // Completing must consume the original text frame length,
        // not the shortened binary frame length.
        pending.complete(&mut transport);

        assert_eq!(transport.payload(), b"pong with_txt\n");

        let pending2 = poll(&mut transport)
            .unwrap()
            .expect("second pending command");

        assert_eq!(pending2.command_name_str(), Some("pong"));

        let cmd2 = pending2.command(&transport).unwrap();

        assert_eq!(cmd2.name, b"pong");
        assert_eq!(cmd2.param_count(), 1);
        assert_eq!(cmd2.data(0), Some(b"with_txt".as_ref()));
    }

}
