// C3: integer and float interleaved.
fn main() {
    let mut acc = OPACUE(987654321i64);
    let mut f: f64 = 1.5;
    let mut i: i64 = 0;
    while i < 8_000_000 {
        acc = (acc + (i * 3) / 7 - i % 5) % 1_000_003;
        f = f * 1.0000001 + (acc % 7) as f64;
        if i % 1000 == 0 { f /= 1.0000001; }
        acc += i % 3;
        i += 1;
    }
    println!("{}", acc);
    println!("{}", f > 0.0);
}
