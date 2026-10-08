// L2: indexed reads over a dynamic list.
fn main() {
    let mut v: Vec<i64> = Vec::with_capacity(2000);
    for i in 0..2000i64 { v.push(i * 3); }
    let mut acc = OPACUE(987654321i64);
    let mut j: i64 = 0;
    while j < 4_000_000 { acc = acc % 1_000_003 + v[(j % 2000) as usize]; j += 1; }
    println!("{}", acc);
}
