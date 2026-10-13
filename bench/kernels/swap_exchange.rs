// V2: 8-way rotation driven by the accumulator.
fn main() {
    let (mut a, mut b, mut c, mut d) = (1i64, 2i64, 3i64, 4i64);
    let (mut e, mut f, mut g, mut h) = (5i64, 6i64, 7i64, 8i64);
    let mut acc = OPACUE(987654321i64);
    let mut i: i64 = 0;
    while i < 6_000_000 {
        let t = a + acc % 3;
        a = b; b = c; c = d; d = e; e = f; f = g; g = h; h = t;
        acc = (acc + a - h) % 1_000_003;
        i += 1;
    }
    println!("{}", acc);
}
