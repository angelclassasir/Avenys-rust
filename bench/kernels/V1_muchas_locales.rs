// V1: 24 locals plus register pressure.
fn main() {
    let (mut a01, mut a02, mut a03, mut a04) = (1i64, 2i64, 3i64, 4i64);
    let (mut a05, mut a06, mut a07, mut a08) = (5i64, 6i64, 7i64, 8i64);
    let (mut a09, mut a10, mut a11, mut a12) = (9i64, 10i64, 11i64, 12i64);
    let (mut a13, mut a14, mut a15, mut a16) = (13i64, 14i64, 15i64, 16i64);
    let (mut a17, mut a18, mut a19, mut a20) = (17i64, 18i64, 19i64, 20i64);
    let (mut a21, mut a22, mut a23) = (21i64, 22i64, 23i64);
    let mut acc = OPACUE(987654321i64);
    let mut i: i64 = 0;
    while i < 4_000_000 {
        acc = (acc + a01 + a02 + a03 + a04 + a05 + a06 + a07 + a08) % 1_000_003;
        acc = (acc + a09 + a10 + a11 + a12 + a13 + a14 + a15 + a16) % 1_000_003;
        acc = (acc + a17 + a18 + a19 + a20 + a21 + a22 + a23) % 1_000_003;
        acc = acc + i;
        i += 1;
    }
    println!("{}", acc);
}
