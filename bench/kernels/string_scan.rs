// A7: substring search over a large text.
fn main() {
    let mut seed = std::hint::black_box(987654321i64);
    let unit = "the quick brown fox jumps over the lazy dog. ";
    let mut haystack = String::with_capacity(unit.len() * 20000);
    for _ in 0..20000 { haystack.push_str(unit); }
    let needle = "lazy dog";
    let mut round: i64 = 0;
    while round < 20 {
        let mut hits: i64 = 0;
        let mut pos: usize = 0;
        while let Some(found) = haystack[pos..].find(needle) {
            hits += 1;
            pos += found + 1;
        }
        seed = (seed + hits) % 1000003;
        round += 1;
    }
    println!("{}", seed);
}
