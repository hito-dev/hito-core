// A simple payload buffer implementation for embedded systems - it can be heap, stack or static
// allocated. It provides methods for managing the payload data and locking mechanism to prevent
// modification by hardware access layer when locked.
pub struct PayloadBuffer<B> {
    buf: B,
    len: usize,
    locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Locked,
    Overflow,
    PayloadChanged,
    NotInitialized,
}

// PayloadBuffer implementation
impl<B> PayloadBuffer<B> 
where 
    B: AsRef<[u8]> + AsMut<[u8]>,
{
    pub fn new(buf: B) -> Self {
        Self {
            buf,
            len: 0,
            locked: false,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf.as_ref()[..self.len]
    }

    pub fn push(&mut self, data: &[u8]) -> Result<(), Error> {
        if self.locked {
            return Err(Error::Locked);
        }

        if self.len + data.len() > self.buf.as_ref().len() {
            return Err(Error::Overflow);
        }
        let end = self.len + data.len();
        self.buf.as_mut()[self.len..end].copy_from_slice(data);
        self.len = end;

        Ok(())
    } 

    pub fn set_len(&mut self, len: usize) {
        assert!(len <= self.capacity());
        self.len = len;
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn consume(&mut self, n: usize) {
        if n >= self.len {
            self.len = 0;
        } else {
            let remaining = self.len - n;
            self.buf.as_mut().copy_within(n..self.len, 0);
            self.len = remaining;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.buf.as_ref().len()
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn as_mut(&mut self) -> Option<&mut [u8]> {
        if self.locked {
            None
        } else {
            Some(self.buf.as_mut())
        }
    }

    pub fn storage_mut(&mut self) -> &mut [u8] {
        self.buf.as_mut()
    }

}

