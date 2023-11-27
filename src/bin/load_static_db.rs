#[path = "../v4.rs"]
mod v4;

fn main() {
    let _md = v4::metadata_for_country_code(353).expect("Ireland should exist");
}
