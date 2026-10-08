// C2: allocation pressure.
fn main() {
    let mut acc = OPACUE(987654321i64);
    let mut i: i64 = 0;
    while i < 8_000_000 {
        let mut v: Vec<i64> = Vec::with_capacity(8);
        for j in 0..8i64 { v.push(j + i); }
        acc = (acc + v.len() as i64 + v[0]) % 1_000_003;
        i += 1;
    }
    println!("{}", acc);
}
