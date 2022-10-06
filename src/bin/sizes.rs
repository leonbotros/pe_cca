//! This file produces a binary that prints the sizes of several IBE/KEM components
//! such as the MPK, MSK, USK, CT and MSG.

macro_rules! print_sizes_kem {
    ($scheme_name: ident) => {{
        use cca_transforms::kem::$scheme_name::*;
        println!(stringify!($scheme_name));
        println!("MPK:\t{}", PK_BYTES);
        println!("MSK:\t{}", SK_BYTES);
        println!("USK:\t{}", USK_BYTES);
        println!("CT:\t{}\n", CT_BYTES);
    }};
}

macro_rules! print_sizes_pke {
    ($scheme_name: ident) => {{
        use cca_transforms::pke::$scheme_name::*;
        println!(stringify!($scheme_name));
        println!("MPK:\t{}", PK_BYTES);
        println!("MSK:\t{}", SK_BYTES);
        println!("USK:\t{}", USK_BYTES);
        println!("CT:\t{}", CT_BYTES);
        println!("MSG:\t{}\n", MSG_BYTES);
    }};
}

fn main() {
    println!("KEM sizes in bytes:\n");
    print_sizes_kem!(cgw_fo);
    print_sizes_kem!(cgw_kv1);
    println!("PKE sizes in bytes:\n");
    print_sizes_pke!(cgw);
}
