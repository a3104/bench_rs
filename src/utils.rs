use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn generate_luid() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_random_hex_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| format!("{:x}", rng.gen_range(0..16))).collect::<String>()
}

pub fn generate_random_number_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| rng.gen_range(0..10).to_string()).collect::<String>()
}

pub fn generate_random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| rng.sample(Alphanumeric) as char).collect::<String>()
}

pub fn replace_random_strings(input: &str) -> String {
    input.replace("{R}", &generate_random_string(10))
}
