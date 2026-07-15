use core::fmt::Display;

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Exception {
    Division = 0,
    Debug = 1,
    Nmi = 2,
    Breakpoint = 3,
    Overflow = 4,
    BoundRangeExceeded = 5,
    InvalidOpcode = 6,
    DeviceNotAvailable = 7,
    DoubleFault = 8,
    CoprocessorSegmentOverrun = 9,
    InvalidTss = 10,
    SegmentNotPresent = 11,
    StackSegmentFault = 12,
    GeneralProtectionFault = 13,
    PageFault = 14,
    Reserved15 = 15,
    FloatingPoint = 16,
    AlignmentCheck = 17,
    MachineCheck = 18,
    SimdFloatingPoint = 19,
    Virtualization = 20,
    ControlProtection = 21,
    Reserved22 = 22,
    Reserved23 = 23,
    Reserved24 = 24,
    Reserved25 = 25,
    Reserved26 = 26,
    Reserved27 = 27,
    HypervisorInjection = 28,
    VmmCommunication = 29,
    Security = 30,
    Reserved31 = 31,
}

impl Display for Exception {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Division => "Division",
            Self::Debug => "Debug",
            Self::Nmi => "Non-Maskable Interrupt",
            Self::Breakpoint => "Breakpoint",
            Self::Overflow => "Overflow",
            Self::BoundRangeExceeded => "Bound Range Exceeded",
            Self::InvalidOpcode => "Invalid Opcode",
            Self::DeviceNotAvailable => "Device Not Available",
            Self::DoubleFault => "Double Fault",
            Self::CoprocessorSegmentOverrun => "Coprocessor Segment Overrun",
            Self::InvalidTss => "Invalid TSS",
            Self::SegmentNotPresent => "Segment Not Present",
            Self::StackSegmentFault => "Stack Segment Fault",
            Self::GeneralProtectionFault => "General Protection Fault",
            Self::PageFault => "Page Fault",
            Self::FloatingPoint => "x87 Floating Point",
            Self::AlignmentCheck => "Alignment Check",
            Self::MachineCheck => "Machine Check",
            Self::SimdFloatingPoint => "SIMD Floating Point",
            Self::Virtualization => "Virtualization",
            Self::ControlProtection => "Control Protection",
            Self::HypervisorInjection => "Hypervisor Injection",
            Self::VmmCommunication => "VMM Communication",
            Self::Security => "Security",
            Self::Reserved15
            | Self::Reserved22
            | Self::Reserved23
            | Self::Reserved24
            | Self::Reserved25
            | Self::Reserved26
            | Self::Reserved27
            | Self::Reserved31 => "Unknown",
        })
    }
}
