fn main() {
    let mut acc = OPACUE(987654321i64);
    for i in 0..8000i64 {
        for j in 0..8000i64 { acc += (i ^ j) % 11; }
    }
    println!("{}", acc);
}
