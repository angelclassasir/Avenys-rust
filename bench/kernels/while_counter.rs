fn main() {
    let mut acc = OPACUE(987654321i64);
    let mut i: i64 = 0;
    while i < 40_000_000 { acc += i % 7; i += 1; }
    println!("{}", acc);
}
