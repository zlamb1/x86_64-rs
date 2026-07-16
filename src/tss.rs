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

    pub const fn address(&self) -> u64 {
        self.lo as u64 | (self.hi as u64) << 32
    }

    pub const fn set_address(&mut self, address: u64) {
        self.lo = address as u32;
        self.hi = (address >> 32) as u32;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IoBitmap<const N: usize> {
    bitmap: [u8; N],
    extra: u8,
}

impl<const N: usize> IoBitmap<N> {
    pub const fn new() -> Self {
        const {
            assert!(N <= 8192, "I/O bitmap only supports 65536 ports.");
        }

        Self {
            bitmap: [0xFF; _],
            extra: 0xFF,
        }
    }

    pub const fn port_enabled(&self, port: u16) -> bool {
        assert!((port as usize) < N * 8);
        let byte = port / 8;
        let bit = port % 8;
        self.bitmap[byte as usize] & 1u8 << bit == 0
    }

    pub const fn port_enable(&mut self, port: u16) {
        assert!((port as usize) < N * 8);
        let byte = port / 8;
        let bit = port % 8;
        self.bitmap[byte as usize] &= !(1u8 << bit);
    }

    pub const fn port_enable_all(&mut self) {
        let mut i = 0;
        while i < N {
            self.bitmap[i] = 0;
            i += 1;
        }
    }

    pub const fn port_disable(&mut self, port: u16) {
        assert!((port as usize) < N * 8);
        let byte = port / 8;
        let bit = port % 8;
        self.bitmap[byte as usize] |= 1u8 << bit;
    }

    pub const fn port_disable_all(&mut self) {
        let mut i = 0;
        while i < N {
            self.bitmap[i] = 0xFF;
            i += 1;
        }
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
    io_bitmap: IoBitmap<N>,
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

    pub const fn port_enabled(&self, port: u16) -> bool {
        self.io_bitmap.port_enabled(port)
    }

    pub const fn port_enable(&mut self, port: u16) {
        self.io_bitmap.port_enable(port);
    }

    pub const fn port_enable_all(&mut self) {
        self.io_bitmap.port_enable_all();
    }

    pub const fn port_disable(&mut self, port: u16) {
        self.io_bitmap.port_disable(port);
    }

    pub const fn port_disable_all(&mut self) {
        self.io_bitmap.port_disable_all();
    }
}
