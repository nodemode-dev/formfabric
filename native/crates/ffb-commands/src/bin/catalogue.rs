fn main() {
    println!("{}", serde_json::to_string_pretty(&ffb_commands::catalogue()).expect("catalogue serializes"));
}
