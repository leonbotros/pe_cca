use criterion::{black_box, criterion_group, criterion_main, Criterion};
use paste::paste;
use std::time::Duration;

macro_rules! bench_kem {
    ($scheme: ident, $struct: ident) => {
        paste! {
            fn [<bench_kem_ $scheme>](criterion: &mut Criterion) {
                use cca_transforms::kem::$scheme::*;
                use cca_transforms::{kem::IBKEM, Derive};

                let mut rng = rand::thread_rng();

                let id = "email:w.geraedts@sarif.nl".as_bytes();
                let kid = <$struct as IBKEM>::Id::derive(id);

                let (pk, sk) = $struct::setup(&mut rng);
                let usk = $struct::extract_usk(Some(&pk), &sk, &kid, &mut rng);

                let (c, _k) = $struct::encaps(&pk, &kid, &mut rng);

                criterion.bench_function(
                    &format!("kem_{} setup", stringify!($scheme)).to_string(),
                    |b| {
                        let mut rng = rand::thread_rng();
                        b.iter(|| $struct::setup(&mut rng))
                    },
                );
                criterion.bench_function(
                    &format!("kem_{} extract", stringify!($scheme)).to_string(),
                    move |b| {
                        let mut rng = rand::thread_rng();
                        b.iter(|| {
                            $struct::extract_usk(
                                black_box(Some(&pk)),
                                black_box(&sk),
                                black_box(&kid),
                                &mut rng,
                            )
                        })
                    },
                );
                criterion.bench_function(
                    &format!("kem_{} encaps", stringify!($scheme)).to_string(),
                    move |b| {
                        let mut rng = rand::thread_rng();
                        b.iter(|| $struct::encaps(black_box(&pk), black_box(&kid), &mut rng))
                    },
                );
                criterion.bench_function(
                    &format!("kem_{} decaps", stringify!($scheme)).to_string(),
                    move |b| {
                        b.iter(|| {
                            $struct::decaps(black_box(Some(&pk)), black_box(&usk), black_box(&c))
                        })
                    },
                );
            }
        }
    };
}

macro_rules! bench_ibe {
    ($scheme: ident, $struct: ident) => {
        paste! {
            fn [<bench_ibe_ $scheme>](criterion: &mut Criterion) {
                use group::Group;
                use cca_transforms::pke::$scheme::*;
                use cca_transforms::{pke::IBE, Derive};
                use rand::RngCore;

                let mut rng = rand::thread_rng();

                let id = "email:w.geraedts@sarif.nl".as_bytes();
                let kid = <$struct as IBE>::Id::derive(id);

                let (pk, sk) = $struct::setup(&mut rng);
                let usk = $struct::extract_usk(Some(&pk), &sk, &kid, &mut rng);

                let m = <$struct as IBE>::Msg::random(&mut rng);
                type RngBytes = <$struct as IBE>::RngBytes;
                let mut rand_bytes: RngBytes = [0u8; core::mem::size_of::<RngBytes>()];
                rng.fill_bytes(&mut rand_bytes);

                let c = $struct::encrypt(&pk, &kid, &m, &rand_bytes);

                criterion.bench_function(
                    &format!("ibe_{} setup", stringify!($scheme)).to_string(),
                    |b| {
                        let mut rng = rand::thread_rng();
                        b.iter(|| $struct::setup(&mut rng))
                    },
                );
                criterion.bench_function(
                    &format!("ibe_{} extract", stringify!($scheme)).to_string(),
                    move |b| {
                        let mut rng = rand::thread_rng();
                        b.iter(|| {
                            $struct::extract_usk(
                                black_box(Some(&pk)),
                                black_box(&sk),
                                black_box(&kid),
                                &mut rng,
                            )
                        })
                    },
                );
                criterion.bench_function(
                    &format!("ibe_{} encrypt", stringify!($scheme)).to_string(),
                    move |b| {
                        b.iter(|| {
                            $struct::encrypt(
                                black_box(&pk),
                                black_box(&kid),
                                black_box(&m),
                                black_box(&rand_bytes),
                            )
                        })
                    },
                );
                criterion.bench_function(
                    &format!("ibe_{} decrypt", stringify!($scheme)).to_string(),
                    move |b| b.iter(|| $struct::decrypt(black_box(&usk), black_box(&c))),
                );
            }
        }
    };
}

fn bench_abe_rwac_cca_this_paper(criterion: &mut Criterion) {
    use cca_transforms::kem::rwac::{gen_a, AccessPolicy, RWAC};
    use group::ff::Field;
    use irmaseal_curve::Scalar;

    for n in [1, 10, 100] {
        let mut rng = rand::thread_rng();
        let (mpk, msk) = RWAC::setup(&mut rng);

        let s: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
        let usk_s = RWAC::extract_usk(&msk, &s[..], &mut rng);

        let a = gen_a(n);
        let rho = s.clone();
        let ap = AccessPolicy { a, rho };

        let (ct, _) = RWAC::encaps(&mpk, &ap, &mut rng);

        criterion.bench_function(&format!("RWAC CCA setup, n = {}", n.to_string()), |b| {
            let mut rng = rand::thread_rng();
            b.iter(|| RWAC::setup(&mut rng))
        });
        criterion.bench_function(
            &format!("RWAC CCA extract, n = {}", n.to_string()),
            move |b| {
                let mut rng = rand::thread_rng();
                b.iter(|| {
                    RWAC::extract_usk(black_box(&msk), black_box(&s[..]), black_box(&mut rng))
                })
            },
        );
        criterion.bench_function(
            &format!("RWAC CCA encrypt, n = {}", n.to_string()),
            move |b| b.iter(|| RWAC::encaps(black_box(&mpk), black_box(&ap), black_box(&mut rng))),
        );
        criterion.bench_function(
            &format!("RWAC CCA decrypt, n = {}", n.to_string()),
            move |b| b.iter(|| RWAC::decaps(black_box(&usk_s), black_box(&ct))),
        );
    }
}

fn bench_abe_rwac_cpa(criterion: &mut Criterion) {
    use cca_transforms::kem::rwac_cpa::{gen_a, AccessPolicy, RWACCPA};
    use group::ff::Field;
    use irmaseal_curve::Scalar;

    for n in [1, 10, 100] {
        let mut rng = rand::thread_rng();
        let (mpk, msk) = RWACCPA::setup(&mut rng);

        let s: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
        let usk_s = RWACCPA::extract_usk(&msk, &s[..], &mut rng);

        let a = gen_a(n);
        let rho = s.clone();
        let ap = AccessPolicy { a, rho };

        let (ct, _) = RWACCPA::encaps(&mpk, &ap, &mut rng);

        criterion.bench_function(&format!("RWAC CPA setup, n = {}", n.to_string()), |b| {
            let mut rng = rand::thread_rng();
            b.iter(|| RWACCPA::setup(&mut rng))
        });
        criterion.bench_function(
            &format!("RWAC CPA extract, n = {}", n.to_string()),
            move |b| {
                let mut rng = rand::thread_rng();
                b.iter(|| {
                    RWACCPA::extract_usk(black_box(&msk), black_box(&s[..]), black_box(&mut rng))
                })
            },
        );
        criterion.bench_function(
            &format!("RWAC CPA encrypt, n = {}", n.to_string()),
            move |b| {
                b.iter(|| RWACCPA::encaps(black_box(&mpk), black_box(&ap), black_box(&mut rng)))
            },
        );
        criterion.bench_function(
            &format!("RWAC CPA decrypt, n = {}", n.to_string()),
            move |b| b.iter(|| RWACCPA::decaps(black_box(&usk_s), black_box(&ct))),
        );
    }
}

/// Estimates cost of CCA by delegability by using RWAC CPA.
fn bench_abe_rwac_cca_del_est(criterion: &mut Criterion) {
    use cca_transforms::kem::rwac_cpa::{gen_a, AccessPolicy, RWACCPA};
    use group::ff::Field;
    use irmaseal_curve::Scalar;

    for n in [1, 10, 100] {
        let mut rng = rand::thread_rng();
        let (mpk, msk) = RWACCPA::setup(&mut rng);

        // setsize + 256
        let s: Vec<Scalar> = (0..n + 256).map(|_| Scalar::random(&mut rng)).collect();
        let usk_s = RWACCPA::extract_usk(&msk, &s[..], &mut rng);

        let a = gen_a(n + 128);
        // attribute size in policy = n + 128
        let rho = s[..n + 128].to_vec();
        let ap = AccessPolicy { a, rho };

        let (ct, _) = RWACCPA::encaps(&mpk, &ap, &mut rng);

        criterion.bench_function(&format!("RWAC del est setup, n = {}", n.to_string()), |b| {
            let mut rng = rand::thread_rng();
            b.iter(|| RWACCPA::setup(&mut rng))
        });
        criterion.bench_function(
            &format!("RWAC del est extract, n = {}", n.to_string()),
            move |b| {
                let mut rng = rand::thread_rng();
                b.iter(|| {
                    RWACCPA::extract_usk(black_box(&msk), black_box(&s[..]), black_box(&mut rng))
                })
            },
        );
        criterion.bench_function(
            &format!("RWAC del est encrypt, n = {}", n.to_string()),
            move |b| {
                b.iter(|| RWACCPA::encaps(black_box(&mpk), black_box(&ap), black_box(&mut rng)))
            },
        );
        criterion.bench_function(
            &format!("RWAC del est decrypt, n = {}", n.to_string()),
            move |b| b.iter(|| RWACCPA::decaps(black_box(&usk_s), black_box(&ct))),
        );
    }
}

/// Estimates cost of CCA by verifiablity by using RWAC CPA.
fn bench_abe_rwac_cca_ver_est(criterion: &mut Criterion) {
    use cca_transforms::kem::rwac_cpa::{gen_a, AccessPolicy, RWACCPA};
    use group::ff::Field;
    use irmaseal_curve::Scalar;

    for n in [1, 10, 100] {
        let mut rng = rand::thread_rng();
        let (mpk, msk) = RWACCPA::setup(&mut rng);

        // setsize remains the same, see actual set passed to extract_usk
        let s: Vec<Scalar> = (0..n + 1).map(|_| Scalar::random(&mut rng)).collect();
        let usk_s = RWACCPA::extract_usk(&msk, &s[..], &mut rng);

        // access policy grows by one
        let a = gen_a(n + 1);
        let rho = s[..].to_vec();
        let ap = AccessPolicy { a, rho };

        let (ct, _) = RWACCPA::encaps(&mpk, &ap, &mut rng);

        criterion.bench_function(&format!("RWAC ver est setup, n = {}", n.to_string()), |b| {
            let mut rng = rand::thread_rng();
            b.iter(|| RWACCPA::setup(&mut rng))
        });
        criterion.bench_function(
            &format!("RWAC ver est extract, n = {}", n.to_string()),
            move |b| {
                let mut rng = rand::thread_rng();
                b.iter(|| {
                    RWACCPA::extract_usk(black_box(&msk), black_box(&s[..n]), black_box(&mut rng))
                })
            },
        );
        criterion.bench_function(
            &format!("RWAC ver est encrypt, n = {}", n.to_string()),
            move |b| {
                b.iter(|| RWACCPA::encaps(black_box(&mpk), black_box(&ap), black_box(&mut rng)))
            },
        );
        criterion.bench_function(
            &format!("RWAC ver est decrypt, n = {}", n.to_string()),
            move |b| {
                b.iter(|| {
                    // verifiability requires 2 decryptions essentially
                    RWACCPA::decaps(black_box(&usk_s), black_box(&ct)).unwrap();
                    RWACCPA::decaps(black_box(&usk_s), black_box(&ct)).unwrap();
                })
            },
        );
    }
}

bench_kem!(cgw_kv1, CGWKV1);
bench_kem!(cgw_fo, CGWFO);
bench_ibe!(cgw, CGW);

criterion_group!(
    name = kem_benches;
    config = Criterion::default().warm_up_time(Duration::new(0, 500));
    targets =
    bench_kem_cgw_fo,
    bench_kem_cgw_kv1,
);

criterion_group!(
    name = pke_benches;
    config = Criterion::default().warm_up_time(Duration::new(0, 500));
    targets =
    bench_ibe_cgw,
);

criterion_group!(
    name = abe_benches;
    config = Criterion::default().warm_up_time(Duration::new(0, 500)).sample_size(10);
    targets =
    bench_abe_rwac_cpa,
    bench_abe_rwac_cca_this_paper,
    bench_abe_rwac_cca_del_est,
    bench_abe_rwac_cca_ver_est,
);

criterion_main!(kem_benches, abe_benches);
