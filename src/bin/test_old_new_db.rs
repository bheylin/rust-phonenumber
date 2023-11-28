#[path = "../v4.rs"]
mod v4;

fn main() {
    let db = &*phonenumber::metadata::DATABASE;
    let phonenumber::metadata::Database {
        cache: _,
        by_id,
        by_code,
        regions,
    } = db.clone();

    for (id, old_metadata) in by_id {
        let new_metadata =
            v4::metadata_for_country_id(&id).expect("metadata should exist for country_id: {id}");

        assert_eq!(old_metadata.id, new_metadata.id);
        assert_eq!(old_metadata.country_code, new_metadata.country_code);
    }

    for (code, old_metadata_list) in by_code {
        let new_metadata_list = v4::metadata_for_country_code(code)
            .expect("metadata should exist for country_code: {code}");

        assert_eq!(old_metadata_list.len(), new_metadata_list.len());

        for old in old_metadata_list {
            assert!(new_metadata_list.iter().find(|m| m.id == old.id).is_some());
        }
    }

    for (country_code, old_regions) in regions {
        let new_regions = v4::regions_for_country_code(country_code)
            .expect("regions should exist for country_code: {country_code}");

        assert_eq!(old_regions.len(), new_regions.len());

        for old in old_regions {
            assert!(new_regions.contains(&&*old));
        }
    }
}
