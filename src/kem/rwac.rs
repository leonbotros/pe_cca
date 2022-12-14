//! IND-ID-CCA2 secure KEM from ABE from Rouselakis and Waters (RW13).

use crate::kem::{Error, SharedSecret};
use crate::util::*;
use alloc::vec::Vec;
use irmaseal_curve::{
    multi_miller_loop, pairing, G1Affine, G1Projective, G2Affine, G2Prepared, G2Projective, Gt,
    Scalar,
};
use rand::{CryptoRng, Rng};

pub type LSSSMatrix = Vec<Vec<Scalar>>;

#[derive(Clone)]
pub struct AccessPolicy {
    pub a: LSSSMatrix,
    pub rho: Vec<Scalar>,
}

/// Generates an LSSS matrix of size n for AND-policies.
pub fn gen_a(n: usize) -> LSSSMatrix {
    if n == 1 {
        vec![vec![Scalar::one()]]
    } else {
        let mut a_mat = Vec::new();

        // 1, 1, 0, 0, ..
        let mut v = vec![Scalar::default(); n];
        v[0] = Scalar::one();
        v[1] = Scalar::one();
        a_mat.push(v);

        for i in 1..n - 1 {
            // 0, 0, -1, 1, 0, 0, etc.
            v = vec![Scalar::default(); n];
            v[i] = Scalar::one().neg();
            v[i + 1] = Scalar::one();
            a_mat.push(v);
        }

        // 0, 0, ..,  -1
        v = vec![Scalar::default(); n];
        v[n - 1] = Scalar::one().neg();
        a_mat.push(v);

        a_mat
    }
}

/// Public key parameters generated by the PKG used to encaps messages.
/// Also known as MPK.
#[derive(Clone, PartialEq)]
pub struct PublicKey {
    /// A
    a: Gt,
    /// g_i
    g: [G1Affine; 2],
    /// Bi
    b: [G1Affine; 2],
    /// B'i
    bprime: [G1Affine; 2],
    /// Bl,i
    b_mat: [[G1Affine; 2]; 2],
    /// B'l,i
    bprime_mat: [[G1Affine; 2]; 2],
}

/// Secret key parameter generated by the PKG used to extract user secret keys.
/// Also known as MSK.
#[derive(Clone, Debug, PartialEq)]
pub struct SecretKey {
    /// alpha
    alpha: [Scalar; 2],
    /// d
    d: [Scalar; 5],
    /// b_i
    b: [Scalar; 3],
    /// b'_i
    bprime: [Scalar; 3],
    /// b_0,i and b_1,i
    b_mat: [[Scalar; 3]; 2],
    /// b'_0,i and b'_1,i
    bprime_mat: [[Scalar; 3]; 2],
}

/// User secret key. Can be used to decaps the corresponding ciphertext.
/// Also known as USK_{S}.
#[derive(Clone)]
pub struct UserSecretKey {
    /// K_i
    k0: [G2Affine; 2],
    /// K'_i
    k1: [G2Affine; 2],
    /// K_i(2)
    k2: [G2Affine; 2],
    /// K_i(3)
    k3: [G2Affine; 2],
    /// K_1,att,1
    k1_attrs: Vec<[G2Affine; 2]>,
    /// K_2,att,1
    k2_attrs: Vec<[G2Affine; 2]>,
    /// S, set of attributes
    attrs: Vec<Scalar>,
}

/// Encrypted message. Can only be decapsed with a corresponding user secret key.
/// Also known as CT_{A}
#[derive(Clone)]
pub struct CipherText {
    /// C'_i
    c0: [G1Affine; 2],
    /// C1,i,j
    c1: [Vec<G1Affine>; 2],
    /// C2,i,j
    c2: [Vec<G1Affine>; 2],
    /// C3,i,j
    c3: [Vec<G1Affine>; 2],
    /// C4,i
    c4: [G1Affine; 2],
    /// k
    k: [u8; 32],
    /// Ap = (A, rho)
    ap: AccessPolicy,
}

#[derive(Clone)]
pub struct RWAC;

impl RWAC {
    /// Generate a keypair used by the Private Key Generator (PKG).
    pub fn setup<R: Rng + CryptoRng>(rng: &mut R) -> (PublicKey, SecretKey) {
        let g = G1Affine::generator();
        let h = G2Affine::generator();

        let alpha = [rand_scalar(rng), rand_scalar(rng)];

        let d = loop {
            let d1 = rand_scalar(rng);
            let d2 = rand_scalar(rng);
            let d3 = rand_scalar(rng);
            let d4 = rand_scalar(rng);

            if d1 * d4 != d2 * d3 {
                let d5 = rand_scalar(rng);
                break [d1, d2, d3, d4, d5];
            }
        };

        let b = [rand_scalar(rng), rand_scalar(rng), rand_scalar(rng)];
        let bprime = [rand_scalar(rng), rand_scalar(rng), rand_scalar(rng)];

        let b_mat = [
            [rand_scalar(rng), rand_scalar(rng), rand_scalar(rng)],
            [rand_scalar(rng), rand_scalar(rng), rand_scalar(rng)],
        ];

        let bprime_mat = [
            [rand_scalar(rng), rand_scalar(rng), rand_scalar(rng)],
            [rand_scalar(rng), rand_scalar(rng), rand_scalar(rng)],
        ];

        let a_pub = pairing(&g, &h) * (alpha[0] * d[0] + alpha[1] * d[1]);

        let mut g_pub = [G1Affine::default(); 2];
        let mut b_pub = [G1Affine::default(); 2];
        let mut bprime_pub = [G1Affine::default(); 2];
        let mut b_mat_pub = [[G1Affine::default(); 2]; 2];
        let mut bprime_mat_pub = [[G1Affine::default(); 2]; 2];

        for i in 0..2 {
            g_pub[i] = (g * d[i]).into();
            b_pub[i] = (g * (b[0] * d[i] + b[2] * d[i + 2])).into();
            bprime_pub[i] = (g * (bprime[0] * d[i] + bprime[2] * d[i + 2])).into();

            for l in 0..2 {
                b_mat_pub[l][i] = (g * (b_mat[l][0] * d[i] + b_mat[l][2] * d[i + 2])).into();
                bprime_mat_pub[l][i] =
                    (g * (bprime_mat[l][0] * d[i] + bprime_mat[l][2] * d[i + 2])).into();
            }
        }

        (
            PublicKey {
                a: a_pub,
                g: g_pub,
                b: b_pub,
                bprime: bprime_pub,
                b_mat: b_mat_pub,
                bprime_mat: bprime_mat_pub,
            },
            SecretKey {
                alpha,
                d,
                b,
                bprime,
                b_mat,
                bprime_mat,
            },
        )
    }

    /// Extract a user secret key for a set of attributes.
    pub fn extract_usk<R: Rng + CryptoRng>(
        sk: &SecretKey,
        attrs: &[Scalar],
        rng: &mut R,
    ) -> UserSecretKey {
        let h = G2Affine::generator();
        let r = rand_scalar(rng);

        let mut alpha = [[Scalar::default(); 2]; 2];
        alpha[0][0] = rand_scalar(rng);
        alpha[1][0] = rand_scalar(rng);
        alpha[0][1] = sk.alpha[0] - alpha[0][0];
        alpha[1][1] = sk.alpha[1] - alpha[1][0];

        // some precalculations
        let d6 = sk.d[4] * (sk.d[0] * sk.d[3] - sk.d[1] * sk.d[2]).invert().unwrap(); // cannot panic, see sampling in setup

        let bbar = [
            d6 * (sk.b[0] * sk.d[3] - sk.b[1] * sk.d[1]),
            d6 * (-sk.b[0] * sk.d[2] + sk.b[1] * sk.d[0]),
        ];

        let bprimebar = [
            d6 * (sk.bprime[0] * sk.d[3] - sk.bprime[1] * sk.d[1]),
            d6 * (-sk.bprime[0] * sk.d[2] + sk.bprime[1] * sk.d[0]),
        ];

        let bbar_mat = [
            [
                // l = 0
                d6 * (sk.b_mat[0][0] * sk.d[3] - sk.b_mat[0][1] * sk.d[1]),
                d6 * (-sk.b_mat[0][0] * sk.d[2] + sk.b_mat[0][1] * sk.d[0]),
            ],
            [
                // l = 1
                d6 * (sk.b_mat[1][0] * sk.d[3] - sk.b_mat[1][1] * sk.d[1]),
                d6 * (-sk.b_mat[1][0] * sk.d[2] + sk.b_mat[1][1] * sk.d[0]),
            ],
        ];

        let bprimebar_mat = [
            [
                // l = 0
                d6 * (sk.bprime_mat[0][0] * sk.d[3] - sk.bprime_mat[0][1] * sk.d[1]),
                d6 * (-sk.bprime_mat[0][0] * sk.d[2] + sk.bprime_mat[0][1] * sk.d[0]),
            ],
            [
                // l = 1
                d6 * (sk.bprime_mat[1][0] * sk.d[3] - sk.bprime_mat[1][1] * sk.d[1]),
                d6 * (-sk.bprime_mat[1][0] * sk.d[2] + sk.bprime_mat[1][1] * sk.d[0]),
            ],
        ];

        // Compute all static key components, k0 through k2
        // K_i
        let k0 = [
            (h * (alpha[0][0] - r * bbar[0])).into(),
            (h * (alpha[1][0] - r * bbar[1])).into(),
        ];

        // K'_i
        let k1 = [
            (h * (r * sk.d[3] * d6)).into(),
            (h * (-r * sk.d[2] * d6)).into(),
        ];

        // K_i(2)
        let k2 = [
            (h * (alpha[0][1] - r * bprimebar_mat[0][0])).into(),
            (h * (alpha[1][1] - r * bprimebar_mat[0][1])).into(),
        ];

        // K_i(3)
        let k3 = [
            (h * (-r * bprimebar_mat[1][0])).into(),
            (h * (-r * bprimebar_mat[1][1])).into(),
        ];

        // Compute all dynamic key components, based on the number of attributes
        // K_1,att,i
        let mut k1_attrs = Vec::<[G2Affine; 2]>::new();
        // K_2,att,i
        let mut k2_attrs = Vec::<[G2Affine; 2]>::new();

        for attr in attrs {
            let r_att = rand_scalar(rng);

            k1_attrs.push([
                (h * (-r_att * (bbar_mat[1][0] * attr + bbar_mat[0][0]) - r * bprimebar[0])).into(), // i = 0
                (h * (-r_att * (bbar_mat[1][1] * attr + bbar_mat[0][1]) - r * bprimebar[1])).into(), // i = 1
            ]);

            // minimal speedup possible:
            // can save 2 scalar multiplicatons here
            k2_attrs.push([
                (h * (r_att * sk.d[3] * d6)).into(),  // K_2,att,1
                (h * (-r_att * sk.d[2] * d6)).into(), // K_2,att,2
            ]);
        }

        UserSecretKey {
            k0,
            k1,
            k2,
            k3,
            k1_attrs,
            k2_attrs,
            attrs: attrs.to_vec(),
        }
    }

    pub fn encaps<R: Rng + CryptoRng>(
        pk: &PublicKey,
        ap: &AccessPolicy,
        rng: &mut R,
    ) -> (CipherText, SharedSecret) {
        // assumes all rows of A are the same length..
        // also assumes a has at least 1 row
        let n1 = ap.a.len();
        let n2 = ap.a[0].len();

        let s = rand_scalar(rng);
        let k = pk.a * s;

        // s_j in paper
        let s_vec: Vec<Scalar> = (0..n1).map(|_| rand_scalar(rng)).collect();

        // v_j' = [v_0, v_1, ..., v_n2-1] with v_0 = 0
        let v: Vec<Scalar> = (0..n2)
            .map(|j| {
                if j == 0 {
                    Scalar::default()
                } else {
                    rand_scalar(rng)
                }
            })
            .collect();

        let ??: Vec<Scalar> =
            ap.a.iter()
                .map(|r| {
                    r.iter()
                        .zip(v.iter())
                        .fold(Scalar::default(), |a, (x, y)| a + x * y)
                })
                .collect();

        let c0 = [(pk.g[0] * s).into(), (pk.g[1] * s).into()];

        let c1: [Vec<G1Affine>; 2] = [
            (0..n1)
                .map(|j| {
                    (pk.b[0] * (ap.a[j][0] * s) + pk.g[0] * ??[j] + pk.bprime[0] * s_vec[j]).into()
                })
                .collect(),
            (0..n1)
                .map(|j| {
                    (pk.b[1] * (ap.a[j][0] * s) + pk.g[1] * ??[j] + pk.bprime[1] * s_vec[j]).into()
                })
                .collect(),
        ];

        let c2: [Vec<G1Affine>; 2] = [
            (0..n1)
                .map(|j| {
                    (pk.b_mat[1][0] * (s_vec[j] * ap.rho[j]) + pk.b_mat[0][0] * s_vec[j]).into()
                })
                .collect(),
            (0..n1)
                .map(|j| {
                    (pk.b_mat[1][1] * (s_vec[j] * ap.rho[j]) + pk.b_mat[0][1] * s_vec[j]).into()
                })
                .collect(),
        ];

        let c3: [Vec<G1Affine>; 2] = [
            (0..n1).map(|j| (pk.g[0] * s_vec[j]).into()).collect(),
            (0..n1).map(|j| (pk.g[1] * s_vec[j]).into()).collect(),
        ];

        let mut smallk = [0u8; 32];
        rng.fill_bytes(&mut smallk);

        let mut rpc_input = Vec::<G1Affine>::new();
        for i in 0..2 {
            for j in 0..n1 {
                rpc_input.push(c0[i]);
                rpc_input.push(c1[i][j]);
                rpc_input.push(c2[i][j]);
                rpc_input.push(c3[i][j]);
            }
        }
        let xprime = rpc(&smallk, &rpc_input);

        let c4 = [
            (pk.bprime_mat[1][0] * (xprime * s) + pk.bprime_mat[0][0] * s).into(),
            (pk.bprime_mat[1][1] * (xprime * s) + pk.bprime_mat[0][1] * s).into(),
        ];

        (
            CipherText {
                c0,
                c1,
                c2,
                c3,
                c4,
                k: smallk,
                ap: ap.clone(),
            },
            SharedSecret::from(&k),
        )
    }

    /// Derive the same SharedSecret from the CipherText using a UserSecretKey.
    ///
    /// This operation always implicitly rejects ciphertexts and therefore never errors.
    pub fn decaps(usk: &UserSecretKey, ct: &CipherText) -> Result<SharedSecret, Error> {
        let n1 = ct.ap.a.len();

        let mut rpc_input = Vec::<G1Affine>::new();
        for i in 0..2 {
            for j in 0..n1 {
                rpc_input.push(ct.c0[i]);
                rpc_input.push(ct.c1[i][j]);
                rpc_input.push(ct.c2[i][j]);
                rpc_input.push(ct.c3[i][j]);
            }
        }

        let yprime = rpc(&ct.k, &rpc_input);

        let upsilon: Vec<usize> = (0..n1)
            .filter(|j| usk.attrs.contains(&ct.ap.rho[*j]))
            .collect();

        let mut pairs = Vec::<(G1Affine, G2Prepared)>::new();

        for i in 0..2 {
            pairs.push((
                ct.c0[i],
                G2Prepared::from(G2Affine::from(
                    G2Projective::from(usk.k0[i])
                        + G2Projective::from(usk.k2[i])
                        + (usk.k3[i] * yprime),
                )),
            ));

            pairs.push((
                G1Affine::from(
                    upsilon
                        .iter()
                        .fold(G1Projective::default(), |acc, j| acc + ct.c1[i][*j])
                        + ct.c4[i],
                ),
                G2Prepared::from(usk.k1[i]),
            ));

            for j in upsilon.iter() {
                let idx = usk.attrs.iter().position(|&x| x == ct.ap.rho[*j]).unwrap();
                pairs.push((ct.c2[i][*j], G2Prepared::from(usk.k2_attrs[idx][i])));
                pairs.push((ct.c3[i][*j], G2Prepared::from(usk.k1_attrs[idx][i])));
            }
        }

        let pairs_ref: Vec<(&G1Affine, &G2Prepared)> = pairs.iter().map(|(i, j)| (i, j)).collect();
        let k = multi_miller_loop(&pairs_ref[..]).final_exponentiation();

        Ok(SharedSecret::from(&k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enc_dec() {
        let n = 10;

        let mut rng = rand::thread_rng();
        let (mpk, msk) = RWAC::setup(&mut rng);

        let s: Vec<Scalar> = (0..n).map(|_| rand_scalar(&mut rng)).collect();
        let usk_s = RWAC::extract_usk(&msk, &s[..], &mut rng);

        let a = gen_a(n);
        let mut rho = s.clone();

        rho.reverse(); // should still work
        let ap = AccessPolicy { a, rho };

        let (ct, ss) = RWAC::encaps(&mpk, &ap, &mut rng);

        let ss2 = RWAC::decaps(&usk_s, &ct).unwrap();

        assert_eq!(ss, ss2);
    }
}
