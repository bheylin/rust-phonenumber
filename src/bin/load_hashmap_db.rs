fn main() {
    let _md = phonenumber::metadata::DATABASE
        .by_code(&353)
        .expect("Ireland should exist");
}
