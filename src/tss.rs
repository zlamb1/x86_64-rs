use core::{cmp::min, ops::ControlFlow};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Sp {
    lo: u32,
    hi: u32,
}

impl Sp {
    pub const fn new(address: u64) -> Self {
        Self {
            lo: address as u32,
            hi: (address >> 32) as u32,
        }
    }

    /// Returns the virtual address of the stack pointer.
    pub const fn address(&self) -> u64 {
        self.lo as u64 | (self.hi as u64) << 32
    }

    /// Sets the virtual address of the stack pointer.
    pub const fn set_address(&mut self, address: u64) {
        self.lo = address as u32;
        self.hi = (address >> 32) as u32;
    }
}

#[derive(Debug)]
pub struct PortRange<'a> {
    bitmap: &'a mut [u8],
    start: u32,
    len: u32,
}

impl<'a> PortRange<'a> {
    /// Constructs a port range from a I/O bitmap.
    ///
    /// # Panics
    ///
    /// Panics if the end of the port range exceeds the I/O bitmap length.
    ///
    pub const fn new<const N: usize>(io_bitmap: &'a mut IoBitmap<N>, start: u32, len: u32) -> Self {
        Self::new_with(&mut io_bitmap.bitmap, start, len)
    }

    /// Constructs a port range from a raw slice.
    ///
    /// # Panics
    ///
    /// Panics if the end of the port range exceeds the slice length.
    ///
    pub const fn new_with(bitmap: &'a mut [u8], start: u32, len: u32) -> Self {
        let end = start.checked_add(len).unwrap() as usize;
        assert!(end <= bitmap.len() * 8);
        Self { bitmap, start, len }
    }

    /// Returns the first port in the port range, assuming the length is non-zero.
    pub const fn start(&self) -> u32 {
        self.start
    }

    /// Returns the length of the port range.
    pub const fn len(&self) -> u32 {
        self.len
    }

    fn mask(bit: u8, len: u8) -> u8 {
        ((1u8 << len) - 1) << bit
    }

    fn walk_bytes(
        start: u32,
        len: u32,
        mut f: impl FnMut(usize, u8) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        let bit = (start % 8) as u8;
        let mut byte = start as usize / 8;
        let mut len = len;

        if bit > 0 && len > 0 {
            let partial_len = min(8 - bit as u32, len) as u8;
            f(byte, Self::mask(bit, partial_len))?;
            byte += 1;
            len -= partial_len as u32;
        }

        while len >= 8 {
            f(byte, 0xFF)?;
            byte += 1;
            len -= 8;
        }

        if len > 0 {
            f(byte, Self::mask(0, len as u8))?;
        }

        ControlFlow::Continue(())
    }

    fn over(&mut self, f: impl Fn(&mut u8, u8)) {
        let _ = Self::walk_bytes(self.start, self.len, |byte, mask| {
            f(&mut self.bitmap[byte], mask);
            ControlFlow::Continue(())
        });
    }

    fn over_ref(&self, f: impl Fn(u8, u8) -> bool) -> bool {
        Self::walk_bytes(self.start, self.len, |byte, mask| {
            if !f(self.bitmap[byte], mask) {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        })
        .is_continue()
    }

    /// Revokes access to all ports in the range.
    pub fn revoke(&mut self) {
        self.over(|byte, mask| *byte |= mask);
    }

    /// Returns true if all ports in the range are revoked.
    pub fn is_revoked(&self) -> bool {
        self.over_ref(|byte, mask| byte & mask == mask)
    }

    /// Grants access to all ports in the range.
    pub fn grant(&mut self) {
        self.over(|byte, mask| *byte &= !mask);
    }

    /// Returns true if all ports in the range are granted.
    pub fn is_granted(&self) -> bool {
        self.over_ref(|byte, mask| byte & mask == 0)
    }

    /// Toggles access to all ports in the range.
    pub fn toggle(&mut self) {
        self.over(|byte, mask| *byte ^= mask);
    }

    /// Removes up to `n` ports from the front of the port range.
    pub fn truncate_front(&mut self, n: u32) {
        let truncate = min(n, self.len);
        self.start += truncate;
        self.len -= truncate;
    }

    /// Removes up to `n` ports from the back of the port range.
    pub fn truncate_back(&mut self, n: u32) {
        let truncate = min(n, self.len);
        self.len -= truncate;
    }

    /// Returns true if the port range is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IoBitmap<const N: usize> {
    bitmap: [u8; N],
    extra: u8,
}

impl<const N: usize> IoBitmap<N> {
    /// The max byte size for an I/O bitmap.
    pub const MAX: usize = 8192;

    /// Constructs an I/O bitmap that supports N * 8 ports.
    pub const fn new() -> Self {
        const {
            assert!(N <= Self::MAX, "I/O bitmap exceeds maximum size");
        }

        Self {
            bitmap: [0xFF; _],
            extra: 0xFF,
        }
    }

    /// Yields the full range of ports in this I/O bitmap.
    pub const fn all(&mut self) -> PortRange<'_> {
        PortRange::new(self, 0, N as u32 * 8)
    }

    /// Yields a single port.
    ///
    /// # Panics
    ///
    /// Panics if the port is out of bounds of the I/O bitmap.
    ///
    pub const fn port(&mut self, port: u16) -> PortRange<'_> {
        PortRange::new(self, port as u32, 1)
    }

    /// Yields a range of ports port..port + len.
    ///
    /// # Panics
    ///
    /// Panics if the port range is out of bounds of the I/O bitmap.
    ///
    pub const fn ports(&mut self, port: u16, len: u16) -> PortRange<'_> {
        PortRange::new(self, port as u32, len as u32)
    }

    /// Yields a range of ports over the supplied `range`. If `range` is backwards, an empty [`PortRange`] is always returned.
    ///
    /// # Panics
    ///
    /// Panics if the port range is out of bounds of the I/O bitmap.
    ///
    pub const fn range(&mut self, range: core::range::Range<u16>) -> PortRange<'_> {
        self.ports(range.start, range.end.saturating_sub(range.start))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Tss {
    reserved1: u32,
    rsp: [Sp; 3],
    reserved2: u32,
    reserved3: u32,
    ist: [Sp; 7],
    reserved4: u32,
    reserved5: u32,
    reserved6: u16,
    io_map_base: u16,
}

impl Tss {
    pub const fn new() -> Self {
        Self {
            reserved1: 0,
            rsp: [Sp::new(0); _],
            reserved2: 0,
            reserved3: 0,
            ist: [Sp::new(0); _],
            reserved4: 0,
            reserved5: 0,
            reserved6: 0,
            io_map_base: u16::MAX,
        }
    }

    pub const fn rsp(&self) -> &[Sp; 3] {
        &self.rsp
    }

    pub const fn rsp_mut(&mut self) -> &mut [Sp; 3] {
        &mut self.rsp
    }

    pub const fn ist(&self) -> &[Sp; 7] {
        &self.ist
    }

    pub const fn ist_mut(&mut self) -> &mut [Sp; 7] {
        &mut self.ist
    }

    pub const fn io_map_base(&self) -> u16 {
        self.io_map_base
    }

    pub const fn set_io_map_base(&mut self, io_map_base: u16) {
        self.io_map_base = io_map_base;
    }

    /// Sets the I/O map base immediately after the TSS.
    pub const fn set_io_map_base_seq(&mut self) {
        self.io_map_base = size_of::<Self>() as u16;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TssIoBitmap<const N: usize> {
    tss: Tss,
    pub io_bitmap: IoBitmap<N>,
}

impl<const N: usize> TssIoBitmap<N> {
    pub const fn new() -> Self {
        let mut tss = Tss::new();
        tss.set_io_map_base_seq();

        Self {
            tss,
            io_bitmap: IoBitmap::new(),
        }
    }

    pub const fn rsp(&self) -> &[Sp; 3] {
        self.tss.rsp()
    }

    pub const fn rsp_mut(&mut self) -> &mut [Sp; 3] {
        self.tss.rsp_mut()
    }

    pub const fn ist(&self) -> &[Sp; 7] {
        self.tss.ist()
    }

    pub const fn ist_mut(&mut self) -> &mut [Sp; 7] {
        self.tss.ist_mut()
    }

    pub const fn io_map_base(&self) -> u16 {
        self.tss.io_map_base()
    }
}
