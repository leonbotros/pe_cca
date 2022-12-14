//! IND-ID-CCA2 secure IBKEM Chen, Gay and Wee.
//!  * From: "[Improved Dual System ABE in Prime-Order Groups via Predicate Encodings](https://link.springer.com/chapter/10.1007/978-3-540-79263-5_14)"
//!
//! CCA security due to a generalized approach from Kiltz & Vahlis.
//!  * From: "[CCA2 Secure IBE: Standard Model Efficiency through Authenticated Symmetric Encryption](https://link.springer.com/chapter/10.1007/978-3-540-79263-5_14)"
//!  * Published in: CT-RSA, 2008

use crate::kem::{Error, SharedSecret, IBKEM};
use crate::util::*;
use crate::Compress;
use irmaseal_curve::{
    multi_miller_loop, pairing, G1Affine, G1Projective, G2Affine, G2Prepared, G2Projective, Gt,
    Scalar,
};
use rand::{CryptoRng, Rng};
use subtle::CtOption;

/// Size of the compressed master public key in bytes.
pub const PK_BYTES: usize = 8 * G1_BYTES + GT_BYTES;

/// Size of the compressed master secret key in bytes.
pub const SK_BYTES: usize = 14 * SCALAR_BYTES;

/// Size of the compressed user secret key in bytes.
pub const USK_BYTES: usize = 6 * G2_BYTES;

/// Size of the compressed ciphertext key in bytes.
pub const CT_BYTES: usize = 4 * G1_BYTES;

/// Public key parameters generated by the PKG used to encaps messages.
/// Also known as MPK.
#[derive(Clone, Copy, PartialEq)]
pub struct PublicKey {
    a_1: [G1Affine; 2],
    w0ta_1: [G1Affine; 2],
    w1ta_1: [G1Affine; 2],
    wprime_1: [G1Affine; 2],
    kta_t: Gt,
}

/// Secret key parameter generated by the PKG used to extract user secret keys.
/// Also known as MSK.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SecretKey {
    b: [Scalar; 2],
    k: [Scalar; 2],
    w0: [[Scalar; 2]; 2],
    w1: [[Scalar; 2]; 2],
    wprime: [[Scalar; 2]; 2],
}

/// User secret key. Can be used to decaps the corresponding ciphertext.
/// Also known as USK_{id}.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UserSecretKey {
    d0: [G2Affine; 2], // K_i
    d1: [G2Affine; 2], // K'_i,1
    d2: [G2Affine; 2], // K'_i,2
}

/// Encrypted message. Can only be decapsed with a corresponding user secret key.
/// Also known as CT_{id}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CipherText {
    c0: [G1Affine; 2],
    c1: [G1Affine; 2],
    k: [u8; 32],
}

/// The CGW-KV1 identity-based key encapsulation scheme.
#[derive(Clone)]
pub struct CGWKV1;

impl IBKEM for CGWKV1 {
    const IDENTIFIER: &'static str = "cgwkv1";

    type Pk = PublicKey;
    type Sk = SecretKey;
    type Usk = UserSecretKey;
    type Ct = CipherText;
    type Ss = SharedSecret;
    type Id = Identity;

    const PK_BYTES: usize = PK_BYTES;
    const SK_BYTES: usize = SK_BYTES;
    const USK_BYTES: usize = USK_BYTES;
    const CT_BYTES: usize = CT_BYTES;

    /// Generate a keypair used by the Private Key Generator (PKG).
    fn setup<R: Rng + CryptoRng>(rng: &mut R) -> (PublicKey, SecretKey) {
        let g1 = G1Affine::generator();
        let g2 = G2Affine::generator();

        let a = [rand_scalar(rng), rand_scalar(rng)];
        let b = [rand_scalar(rng), rand_scalar(rng)];

        let w0 = [
            [rand_scalar(rng), rand_scalar(rng)],
            [rand_scalar(rng), rand_scalar(rng)],
        ];

        let w1 = [
            [rand_scalar(rng), rand_scalar(rng)],
            [rand_scalar(rng), rand_scalar(rng)],
        ];

        let wprime = [
            [rand_scalar(rng), rand_scalar(rng)],
            [rand_scalar(rng), rand_scalar(rng)],
        ];

        let k = [rand_scalar(rng), rand_scalar(rng)];

        let w0a = [
            w0[0][0] * a[0] + w0[1][0] * a[1],
            w0[0][1] * a[0] + w0[1][1] * a[1],
        ];
        let w1a = [
            w1[0][0] * a[0] + w1[1][0] * a[1],
            w1[0][1] * a[0] + w1[1][1] * a[1],
        ];
        let wprimea = [
            wprime[0][0] * a[0] + wprime[1][0] * a[1],
            wprime[0][1] * a[0] + wprime[1][1] * a[1],
        ];

        let batch = [
            g1 * a[0],
            g1 * a[1],
            g1 * w0a[0],
            g1 * w0a[1],
            g1 * w1a[0],
            g1 * w1a[1],
            g1 * wprimea[0],
            g1 * wprimea[1],
        ];

        let mut out = [G1Affine::default(); 8];
        G1Projective::batch_normalize(&batch, &mut out);
        let kta_t = pairing(&g1, &g2) * (k[0] * a[0] + k[1] * a[1]);

        (
            PublicKey {
                a_1: [out[0], out[1]],
                w0ta_1: [out[2], out[3]],
                w1ta_1: [out[4], out[5]],
                wprime_1: [out[6], out[7]],
                kta_t,
            },
            SecretKey {
                b,
                k,
                w0,
                w1,
                wprime,
            },
        )
    }

    /// Extract a user secret key for a given identity.
    fn extract_usk<R: Rng + CryptoRng>(
        _pk: Option<&PublicKey>,
        sk: &SecretKey,
        v: &Identity,
        rng: &mut R,
    ) -> UserSecretKey {
        let g2 = G2Affine::generator();
        let r = rand_scalar(rng);
        let id = v.to_scalar();

        let br = [sk.b[0] * r, sk.b[1] * r];

        let batch = [
            g2 * br[0],
            g2 * br[1],
            g2 * (sk.k[0]
                - (br[0] * sk.w0[0][0]
                    + br[1] * sk.w0[0][1]
                    + id * (br[0] * sk.w1[0][0] + br[1] * sk.w1[0][1]))),
            g2 * (sk.k[1]
                - (br[0] * sk.w0[1][0]
                    + br[1] * sk.w0[1][1]
                    + id * (br[0] * sk.w1[1][0] + br[1] * sk.w1[1][1]))),
            g2 * -(br[0] * sk.wprime[0][0] + br[1] * sk.wprime[0][1]),
            g2 * -(br[0] * sk.wprime[1][0] + br[1] * sk.wprime[1][1]),
        ];
        let mut out = [G2Affine::default(); 6];
        G2Projective::batch_normalize(&batch, &mut out);

        UserSecretKey {
            d0: [out[0], out[1]], // K_i
            d1: [out[2], out[3]], // K'_i,0
            d2: [out[4], out[5]], // K'_i,1
        }
    }

    fn encaps<R: Rng + CryptoRng>(
        pk: &PublicKey,
        id: &Identity,
        rng: &mut R,
    ) -> (CipherText, SharedSecret) {
        let s = rand_scalar(rng);
        let k = pk.kta_t * s;

        let x = id.to_scalar();
        let c0 = [(pk.a_1[0] * s).into(), (pk.a_1[1] * s).into()];

        let mut smallk = [0u8; 32];
        rng.fill_bytes(&mut smallk);

        let xprime = rpc(&smallk, &[c0[0], c0[1]]);

        // TODO: can optimize with left-to-right square-and-multiply.
        let c1: [G1Affine; 2] = [
            ((pk.w0ta_1[0] * s) + (pk.w1ta_1[0] * (s * x)) + (pk.wprime_1[0] * (s * xprime)))
                .into(),
            ((pk.w0ta_1[1] * s) + (pk.w1ta_1[1] * (s * x)) + (pk.wprime_1[1] * (s * xprime)))
                .into(),
        ];

        (CipherText { c0, c1, k: smallk }, SharedSecret::from(&k))
    }

    /// Derive the same SharedSecret from the CipherText using a UserSecretKey.
    ///
    /// # Errors
    ///
    /// This operation always implicitly rejects ciphertexts and therefore never errors.
    fn decaps(
        _pk: Option<&PublicKey>,
        usk: &UserSecretKey,
        ct: &CipherText,
    ) -> Result<SharedSecret, Error> {
        let yprime = rpc(&ct.k, &[ct.c0[0], ct.c0[1]]);
        let tmp1: G2Affine = (usk.d1[0] + (usk.d2[0] * yprime)).into();
        let tmp2: G2Affine = (usk.d1[1] + (usk.d2[1] * yprime)).into();

        let m = multi_miller_loop(&[
            (&ct.c0[0], &G2Prepared::from(tmp1)),
            (&ct.c0[1], &G2Prepared::from(tmp2)),
            (&ct.c1[0], &G2Prepared::from(usk.d0[0])),
            (&ct.c1[1], &G2Prepared::from(usk.d0[1])),
        ])
        .final_exponentiation();

        Ok(SharedSecret::from(&m))
    }
}

impl Compress for PublicKey {
    const OUTPUT_SIZE: usize = PK_BYTES;
    type Output = [u8; Self::OUTPUT_SIZE];

    fn to_bytes(&self) -> [u8; PK_BYTES] {
        unimplemented!();
    }

    fn from_bytes(_bytes: &[u8; PK_BYTES]) -> CtOption<Self> {
        unimplemented!();
    }
}

impl Compress for SecretKey {
    const OUTPUT_SIZE: usize = SK_BYTES;
    type Output = [u8; Self::OUTPUT_SIZE];

    fn to_bytes(&self) -> [u8; SK_BYTES] {
        unimplemented!();
    }

    fn from_bytes(_bytes: &[u8; SK_BYTES]) -> CtOption<Self> {
        unimplemented!();
    }
}

impl Compress for UserSecretKey {
    const OUTPUT_SIZE: usize = USK_BYTES;
    type Output = [u8; Self::OUTPUT_SIZE];

    fn to_bytes(&self) -> [u8; USK_BYTES] {
        unimplemented!();
    }

    fn from_bytes(_bytes: &[u8; USK_BYTES]) -> CtOption<Self> {
        unimplemented!();
    }
}

impl Compress for CipherText {
    const OUTPUT_SIZE: usize = CT_BYTES;
    type Output = [u8; Self::OUTPUT_SIZE];

    fn to_bytes(&self) -> [u8; CT_BYTES] {
        unimplemented!();
    }

    fn from_bytes(_bytes: &[u8; CT_BYTES]) -> CtOption<Self> {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Derive;

    test_kem!(CGWKV1);
}
