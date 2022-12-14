//! This module contains key encapsulation mechnisms based on IBE schemes.

#[cfg(feature = "cgwfo")]
#[cfg_attr(docsrs, doc(cfg(feature = "cgwfo")))]
pub mod cgw_fo;

#[cfg(feature = "cgwkv1")]
#[cfg_attr(docsrs, doc(cfg(feature = "cgwkv1")))]
pub mod cgw_kv1;

#[cfg(feature = "rwac")]
#[cfg_attr(docsrs, doc(cfg(feature = "rwac")))]
pub mod rwac;

#[cfg(feature = "rwac_cpa")]
#[cfg_attr(docsrs, doc(cfg(feature = "rwac_cpa")))]
pub mod rwac_cpa;

use crate::util::*;
use crate::{Compress, Derive};
use irmaseal_curve::Gt;
use rand::{CryptoRng, Rng};

/// Size of the shared secret in bytes.
pub const SS_BYTES: usize = 32;

/// All KEMs in this library produce a 64-byte shared secret.
///
/// This shared secret has roughly a 127 bits of security.
/// This is due to the fact that BLS12-381 targets this security level (optimistically).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SharedSecret(pub [u8; SS_BYTES]);

/// Uses SHAKE256 to derive a 64-byte shared secret from a target group element.
///
/// Internally compresses the target group element to byte representation.
impl From<&Gt> for SharedSecret {
    fn from(el: &Gt) -> Self {
        SharedSecret(shake256::<SS_BYTES>(&el.to_compressed()))
    }
}

#[derive(Debug)]
pub enum Error {
    /// Error indicating that the decapsulation was not successful.
    Decapsulation,
    /// Error that the given buffer of ciphertexts is not the correct size.
    IncorrectSize,
}

/// Identity-based key encapsulation mechanism (IBKEM).
pub trait IBKEM: Clone {
    /// Scheme identifier.
    const IDENTIFIER: &'static str;

    /// Master public key (Mpk).
    type Pk: Compress;

    /// Master secret key (Msk).
    type Sk: Compress;

    /// User secret key (Usk).
    type Usk: Compress;

    /// Ciphertext (Ct).
    type Ct: Compress + Default;

    /// Identity.
    type Id: Copy + Default + Derive;

    /// Shared secret.
    type Ss: Copy;

    /// Size of the master public key in bytes.
    const PK_BYTES: usize;

    /// Size of the master secret key in bytes.
    const SK_BYTES: usize;

    /// Size of the user secret key in bytes.
    const USK_BYTES: usize;

    /// Size of the ciphertext in bytes.
    const CT_BYTES: usize;

    /// Creates a MSK, MPK pair.
    fn setup<R: Rng + CryptoRng>(rng: &mut R) -> (Self::Pk, Self::Sk);

    /// Extract a user secret key for an identity using the MSK.
    ///
    /// Optionally requires the system's public key.
    fn extract_usk<R: Rng + CryptoRng>(
        pk: Option<&Self::Pk>,
        sk: &Self::Sk,
        id: &Self::Id,
        rng: &mut R,
    ) -> Self::Usk;

    /// Encapsulate a shared secret using the master public key and an identity.
    fn encaps<R: Rng + CryptoRng>(
        pk: &Self::Pk,
        id: &Self::Id,
        rng: &mut R,
    ) -> (Self::Ct, Self::Ss);

    /// Decrypt a ciphertext using a user secret key to retrieve the shared secret.
    ///
    /// Optionally requires a public key to perform this operation.
    ///
    /// For some schemes this operation can fail explicitly, e.g., when
    /// a bogus ciphertext is used as input.
    fn decaps(mpk: Option<&Self::Pk>, usk: &Self::Usk, ct: &Self::Ct) -> Result<Self::Ss, Error>;
}
