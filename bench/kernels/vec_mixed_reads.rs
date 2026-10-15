// L5: mixed reads over one resident vector.
fn main() {
    let mut seed = OPACUE(987654321i64);
    let mut v: Vec<i64> = Vec::new();
    let mut k: i64 = 0;
    while k < 4096 {
        v.push(k * 5 + 1);
        k += 1;
    }
    let mut j: i64 = 0;
    while j < 12_000_000 {
        let idx = (j * 4093) % 4096;
        seed = (seed + v[idx as usize] + v.len() as i64) % 1_000_003;
        j += 1;
    }
    seed = (seed + v[0] + v[4095]) % 1_000_003;
    println!("{}", seed);
}