// L3: vector growth.
fn main() {
    let mut seed = OPACUE(987654321i64);
    let mut round: i64 = 0;
    while round < 44000 {
        // Starts at capacity 0, so every vector reallocates its way up to 2000.
        let mut v: Vec<i64> = Vec::new();
        let mut k: i64 = 0;
        while k < 2000 {
            v.push(k * 3 + round);
            k += 1;
        }
        seed = (seed + v.len() as i64 + v[1999]) % 1_000_003;
        round += 1;
    }
    println!("{}", seed);
}