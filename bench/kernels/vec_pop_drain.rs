// L4: build then drain with pop.
fn main() {
    let mut seed = OPACUE(987654321i64);
    let mut round: i64 = 0;
    while round < 48000 {
        let mut v: Vec<i64> = Vec::new();
        let mut k: i64 = 0;
        while k < 200 {
            v.push(k * 7 + round);
            k += 1;
        }
        seed = (seed + v.len() as i64) % 1_000_003;
        let mut j: i64 = 0;
        while j < 200 {
            seed = (seed + v.pop().unwrap_or(0) + j) % 1_000_003;
            j += 1;
        }
        seed = (seed + v.len() as i64) % 1_000_003;
        round += 1;
    }
    println!("{}", seed);
}