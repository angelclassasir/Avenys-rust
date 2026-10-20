// A9: number-to-text and back.
fn main() {
    let mut seed = std::hint::black_box(987654321i64);
    let mut i: i64 = 0;
    while i < 2000000 {
        let n = (seed + i) % 1000000;
        let s = n.to_string();
        let back: i64 = s.parse().unwrap();
        seed = (seed + back + s.len() as i64) % 1000003;
        i += 1;
    }
    println!("{}", seed);
}
