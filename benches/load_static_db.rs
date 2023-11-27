#[path = "../src/v4.rs"]
mod v4;

fn main() {
    divan::main();
}

#[divan::bench]
fn get_metadata_for_country_code() {
    let _md = v4::metadata_for_country_code(353).expect("Ireland should exist");
}

#[divan::bench]
fn get_metadata_for_country_id() {
    let _md = v4::metadata_for_country_id("IE").expect("Ireland should exist");
}

#[divan::bench]
fn get_regionsfor_country_code() {
    let _regions = v4::regions_for_country_code(353).expect("Ireland should exist");
}
