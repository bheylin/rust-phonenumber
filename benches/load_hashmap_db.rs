fn main() {
    divan::main();
}

#[divan::bench]
fn get_metadata_for_country_code() {
    let _md = phonenumber::metadata::DATABASE
        .by_code(&353)
        .expect("Ireland should exist");
}

#[divan::bench]
fn get_metadata_for_country_id() {
    let _md = phonenumber::metadata::DATABASE
        .by_id("IE")
        .expect("Ireland should exist");
}

#[divan::bench]
fn get_regionsfor_country_code() {
    let _regions = phonenumber::metadata::DATABASE
        .region(&353)
        .expect("Ireland should exist");
}

#[divan::bench]
fn get_metadata_for_country_code_hot() {
    let _md = phonenumber::metadata::DATABASE
        .by_code(&353)
        .expect("Ireland should exist");
}
