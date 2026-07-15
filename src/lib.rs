#![cfg_attr(not(feature = "std"), no_std)]

pub mod error_code;
pub mod exception;
pub mod idt;
pub mod int;
pub mod msr;
pub mod segment;
pub mod spin;

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Dpl {
    /// Kernel
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    /// User
    Ring3 = 3,
}

impl TryFrom<u8> for Dpl {
    type Error = ();

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        Ok(match x {
            0 => Self::Ring0,
            1 => Self::Ring1,
            2 => Self::Ring2,
            3 => Self::Ring3,
            _ => return Err(()),
        })
    }
}
