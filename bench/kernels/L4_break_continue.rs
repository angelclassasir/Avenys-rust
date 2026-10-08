fn main() {
    let mut acc = OPACUE(987654321i64);
    let mut i: i64 = 0;
    while i < 20_000_000 {
        i += 1;
        if i % 3 == 0 { continue; }
        if i % 1000 == 0 { acc += i; }
        if i % 2 == 0 { acc -= 1; } else { acc += 2; }
    }
    println!("{}", acc);
}
