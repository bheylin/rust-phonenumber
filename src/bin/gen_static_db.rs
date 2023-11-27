use std::collections::HashMap;

use lazy_static::__Deref;
use phonenumber::metadata::Database;
use regex_cache::CachedRegex;

fn main() {
    let db = &*phonenumber::metadata::DATABASE;
    let Database {
        cache: _,
        by_id,
        by_code,
        regions,
    } = db.clone();

    let mut meta_map: HashMap<Metadata, Option<usize>> = HashMap::new();

    eprintln!("Converting by_id HashMap to intermediate Metadata");

    let by_id: HashMap<String, Metadata> = by_id
        .into_iter()
        .map(|(country_id, metadata)| {
            let m = metadata.deref().clone();
            let metadata: Metadata = m.into();
            (country_id, metadata)
        })
        .collect();

    eprintln!("Converting by_code HashMap to intermediate Metadata");

    let by_code: HashMap<u16, Vec<Metadata>> = by_code
        .into_iter()
        .map(|(country_code, metadata_list)| {
            let metadata_list = metadata_list
                .into_iter()
                .map(|m| Metadata::from(m.deref().clone()))
                .collect();

            (country_code, metadata_list)
        })
        .collect();

    eprintln!("Parsing by_id HashMap");

    for meta in by_id.values() {
        meta_map.insert(meta.clone(), None);
    }

    for meta_list in by_code.values() {
        for meta in meta_list {
            meta_map.insert(meta.clone(), None);
        }
    }

    println!("const METADATA: [Metadata; {}] = [", meta_map.len());

    for (i, (metadata, index)) in meta_map.iter_mut().enumerate() {
        index.replace(i);
        println!("    {metadata:#?},");
    }

    println!("];\n");

    eprintln!("Sorting by_code HashMap");

    let mut by_id: Vec<(String, Metadata)> = by_id.into_iter().collect();
    by_id.sort_unstable_by_key(|(country_id, _)| country_id.clone());

    println!("pub fn metadata_for_country_id(country_id: &str) -> Option<&'static Metadata> {{");
    println!("    match country_id {{");

    for (country_id, metadata) in by_id {
        let meta_index = meta_map[&metadata].expect("Index should exist");
        println!("        {country_id:?} => Some(&METADATA[{meta_index:?}]),",);
    }

    println!("        _ => None");
    println!("    }}");
    println!("}}\n");

    let mut by_code: Vec<(u16, Vec<Metadata>)> = by_code.into_iter().collect();
    by_code.sort_unstable_by_key(|(country_code, _)| country_code.clone());

    println!("pub fn metadata_for_country_code(country_code: u16) -> Option<&'static [&'static Metadata]> {{");
    println!("    match country_code {{");
    for (country_code, metadata_list) in by_code {
        print!("        {:?} => Some(&[", country_code);

        for metadata in metadata_list {
            let meta_index = meta_map[&metadata].expect("Index should exist");
            print!("&METADATA[{meta_index}], ");
        }

        println!("]),")
    }
    println!("        _ => None");
    println!("    }}");
    println!("}}\n");

    let mut regions: Vec<(u16, Vec<String>)> = regions.into_iter().collect();
    regions.sort_unstable_by_key(|v| v.0);

    println!(
        "pub fn regions_for_country_code(country_code: u16) -> Option<&'static [&'static str]> {{"
    );
    println!("    match country_code {{");
    for (country_code, regions) in regions {
        println!("        {country_code} => Some(&{regions:?}),");
    }
    println!("        _ => None");
    println!("    }}");
    println!("}}\n");
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Metadata {
    descriptors: Descriptors,
    id: String,
    country_code: u16,

    international_prefix: Option<String>,
    preferred_international_prefix: Option<String>,
    national_prefix: Option<String>,
    preferred_extension_prefix: Option<String>,
    national_prefix_for_parsing: Option<String>,
    national_prefix_transform_rule: Option<String>,

    formats: Vec<Format>,
    international_formats: Vec<Format>,
    main_country_for_code: bool,
    leading_digits: Option<String>,
    mobile_number_portable: bool,
}

/// Descriptors for various types of phone number.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Descriptors {
    general: Descriptor,
    fixed_line: Option<Descriptor>,
    mobile: Option<Descriptor>,
    toll_free: Option<Descriptor>,
    premium_rate: Option<Descriptor>,
    shared_cost: Option<Descriptor>,
    personal_number: Option<Descriptor>,
    voip: Option<Descriptor>,
    pager: Option<Descriptor>,
    uan: Option<Descriptor>,
    emergency: Option<Descriptor>,
    voicemail: Option<Descriptor>,
    short_code: Option<Descriptor>,
    standard_rate: Option<Descriptor>,
    carrier: Option<Descriptor>,
    no_international: Option<Descriptor>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Descriptor {
    national_number: String,

    possible_length: Vec<u16>,
    possible_local_length: Vec<u16>,

    example: Option<String>,
}

/// Description of a phone number format.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Format {
    pattern: String,
    format: String,

    leading_digits: Vec<String>,
    national_prefix: Option<String>,
    national_prefix_optional: bool,
    domestic_carrier: Option<String>,
}

fn regex_to_string(regex: CachedRegex) -> String {
    strip_string(regex.to_string())
}

fn strip_string(s: String) -> String {
    s.replace(&['\n', '\t', ' '], "")
}

impl From<phonenumber::Metadata> for Metadata {
    fn from(value: phonenumber::Metadata) -> Self {
        let phonenumber::Metadata {
            descriptors,
            id,
            country_code,
            international_prefix,
            preferred_international_prefix,
            national_prefix,
            preferred_extension_prefix,
            national_prefix_for_parsing,
            national_prefix_transform_rule,
            formats,
            international_formats,
            main_country_for_code,
            leading_digits,
            mobile_number_portable,
        } = value;

        Self {
            descriptors: descriptors.into(),
            id,
            country_code,
            international_prefix: international_prefix.map(regex_to_string),
            preferred_international_prefix,
            national_prefix,
            preferred_extension_prefix,
            national_prefix_for_parsing: national_prefix_for_parsing.map(regex_to_string),
            national_prefix_transform_rule,
            formats: formats.into_iter().map(Into::into).collect(),
            international_formats: international_formats.into_iter().map(Into::into).collect(),
            main_country_for_code,
            leading_digits: leading_digits.map(regex_to_string),
            mobile_number_portable,
        }
    }
}

impl From<phonenumber::metadata::Format> for Format {
    fn from(value: phonenumber::metadata::Format) -> Self {
        let phonenumber::metadata::Format {
            pattern,
            format,
            leading_digits,
            national_prefix,
            national_prefix_optional,
            domestic_carrier,
        } = value;

        Self {
            pattern: regex_to_string(pattern),
            format,
            leading_digits: leading_digits.into_iter().map(regex_to_string).collect(),
            national_prefix,
            national_prefix_optional,
            domestic_carrier,
        }
    }
}

impl From<phonenumber::metadata::Descriptor> for Descriptor {
    fn from(value: phonenumber::metadata::Descriptor) -> Self {
        let phonenumber::metadata::Descriptor {
            national_number,
            possible_length,
            possible_local_length,
            example,
        } = value;

        Self {
            national_number: regex_to_string(national_number),
            possible_length,
            possible_local_length,
            example,
        }
    }
}

impl From<phonenumber::metadata::Descriptors> for Descriptors {
    fn from(value: phonenumber::metadata::Descriptors) -> Self {
        let phonenumber::metadata::Descriptors {
            general,
            fixed_line,
            mobile,
            toll_free,
            premium_rate,
            shared_cost,
            personal_number,
            voip,
            pager,
            uan,
            emergency,
            voicemail,
            short_code,
            standard_rate,
            carrier,
            no_international,
        } = value;

        Self {
            general: general.into(),
            fixed_line: fixed_line.map(Into::into),
            mobile: mobile.map(Into::into),
            toll_free: toll_free.map(Into::into),
            premium_rate: premium_rate.map(Into::into),
            shared_cost: shared_cost.map(Into::into),
            personal_number: personal_number.map(Into::into),
            voip: voip.map(Into::into),
            pager: pager.map(Into::into),
            uan: uan.map(Into::into),
            emergency: emergency.map(Into::into),
            voicemail: voicemail.map(Into::into),
            short_code: short_code.map(Into::into),
            standard_rate: standard_rate.map(Into::into),
            carrier: carrier.map(Into::into),
            no_international: no_international.map(Into::into),
        }
    }
}
