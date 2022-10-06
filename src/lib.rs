//! Code accompanying "Efficient and Generic Transformations for Chosen-Ciphertext Secure Predicate Encryption"
//!
//! * Chen-Gay-Wee (IND-ID-CPA IBE, IND-ID-CCA2 IBKEM).
//! * RWAC (IND-ID-CPA ABE, IND-ID-CCA2 ABE KEM).
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(test)]
extern crate std;

#[cfg(any(feature = "rwac", feature = "rwac_cpa"))]
#[macro_use]
extern crate alloc;

#[cfg(test)]
#[macro_use]
#[allow(unused)]
mod test_macros;

#[allow(unused)]
mod util;

pub mod kem;
pub mod pke;

/// Artifacts of the system.
///
/// Can be compressed to byte format and back. Each scheme has its own associated types and
/// therefore produce diffently sized byte arrays.
pub trait Compress: Copy {
    const OUTPUT_SIZE: usize;
    type Output: Copy + Clone + AsRef<[u8]>;
    fn to_bytes(self: &Self) -> Self::Output;
    fn from_bytes(output: &Self::Output) -> subtle::CtOption<Self>;
}

pub trait Derive {
    fn derive(b: &[u8]) -> Self;
    fn derive_str(s: &str) -> Self;
}
