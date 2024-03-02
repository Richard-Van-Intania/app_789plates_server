fn main() {
    println!("Hello, world!");
    let hash1 = blake3::hash(b"foobarbaz");
    println!("{}", hash1);
}
