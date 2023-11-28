extern crate lazy_regex;

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::ops::{Deref, Range};

use lazy_regex::regex;
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

    eprintln!("Converting by_id HashMap to intermediate Metadata");

    let by_id: BTreeMap<String, Metadata> = by_id
        .into_iter()
        .map(|(country_id, metadata)| {
            let m = metadata.deref().clone();
            let metadata: Metadata = m.into();
            (country_id, metadata)
        })
        .collect();

    eprintln!("Converting by_code HashMap to intermediate Metadata");

    let by_code: BTreeMap<u16, Vec<Metadata>> = by_code
        .into_iter()
        .map(|(country_code, metadata_list)| {
            let metadata_list = metadata_list
                .into_iter()
                .map(|m| Metadata::from(m.deref().clone()))
                .collect();

            (country_code, metadata_list)
        })
        .collect();

    let regions: BTreeMap<u16, Vec<String>> = regions.into_iter().collect();

    let (by_id, by_code) = print_metadata_cache(by_id, by_code);
    // print_metadata_for_country_id_linear_str_match(by_id, &meta_map);
    print_metadata_for_country_id_nested_byte_match(by_id);
    print_metadata_for_country_code(by_code);
    print_regions_for_country_code(regions);
}

fn print_metadata_cache(
    by_id: BTreeMap<String, Metadata>,
    by_code: BTreeMap<u16, Vec<Metadata>>,
) -> (BTreeMap<String, usize>, BTreeMap<u16, Range<usize>>) {
    eprintln!("Print metadata cache");

    let mut meta_cache = HashMap::<Metadata, usize>::new();
    let mut index = 0;
    let mut buf = String::new();

    let by_code_indexed = by_code
        .into_iter()
        .map(|(country_code, metadata_list)| {
            let start = index;
            for metadata in metadata_list {
                write!(buf, "    {metadata:#?},\n").expect("write failed");
                meta_cache.insert(metadata, index);
                index = index + 1;
            }
            let end = index;
            (country_code, start..end)
        })
        .collect();

    let by_id_indexed = by_id
        .into_iter()
        .map(|(country_id, metadata)| {
            let meta_index = if let Some(i) = meta_cache.get(&metadata) {
                *i
            } else {
                write!(buf, "    {metadata:#?},\n").expect("write failed");
                meta_cache.insert(metadata, index);
                index = index + 1;
                index
            };

            (country_id, meta_index)
        })
        .collect();

    println!("const METADATA: [Metadata; {}] = [", meta_cache.len());
    println!("{buf}");
    println!("];\n");

    (by_id_indexed, by_code_indexed)
}

fn print_metadata_for_country_id_linear_str_match(by_id: BTreeMap<String, usize>) {
    println!("pub fn metadata_for_country_id(country_id: &str) -> Option<&'static Metadata> {{");
    println!("    match country_id {{");

    for (country_id, index) in by_id {
        println!("        {country_id:?} => Some(&METADATA[{index:?}]),",);
    }

    println!("        _ => None");
    println!("    }}");
    println!("}}\n");
}

fn print_metadata_for_country_id_nested_byte_match(by_id: BTreeMap<String, usize>) {
    let (alphabetic, numerical): (Vec<_>, Vec<_>) = by_id
        .into_iter()
        .partition(|(id, _index)| id.chars().all(|c| c.is_ascii_alphabetic()));

    assert_eq!(
        numerical.len(),
        1,
        "Expecting only one numerical code `001`"
    );

    let mut alpha_map: BTreeMap<u8, Vec<(u8, usize)>> = BTreeMap::new();

    for (id, index) in alphabetic {
        let bytes = id.as_bytes();
        let first_byte = bytes[0];
        let second_byte = bytes[1];
        let chars = alpha_map.entry(first_byte).or_insert_with(Vec::new);
        chars.push((second_byte, index));
    }

    println!("pub fn metadata_for_country_id(country_id: &str) -> Option<&'static Metadata> {{");
    println!("    let bytes = country_id.as_bytes();\n");
    println!("    if bytes.len() < 2 || bytes.len() > 3 {{");
    println!("        return None;");
    println!("    }}\n");

    println!("    if bytes == b\"001\" {{");
    println!("        return Some(&METADATA[{}]);", numerical[0].1);
    println!("    }}\n");

    println!("    match bytes[0] {{");

    for (first_byte, post) in alpha_map {
        println!("        b{:?} => match bytes[1] {{", char::from(first_byte));
        for (second_byte, meta_index) in post {
            println!(
                "            b{:?} => Some(&METADATA[{meta_index:?}]),",
                char::from(second_byte)
            );
        }
        println!("            _ => None,");
        println!("        }}");
    }

    println!("        _ => None");
    println!("    }}");
    println!("}}\n");
}

fn print_metadata_for_country_code(by_code: BTreeMap<u16, Range<usize>>) {
    println!("pub fn metadata_for_country_code(country_code: u16) -> Option<&'static [&'static Metadata]> {{");
    println!("    match country_code {{");
    for (country_code, index_range) in by_code {
        if index_range.end - index_range.start == 1 {
            println!(
                "        {country_code:?} => Some(&[&METADATA[{}]]),",
                index_range.start
            );
        } else {
            print!("        {country_code:?} => Some(&[");

            for i in index_range {
                print!("&METADATA[{i}], ")
            }

            println!("]),");
        }
    }
    println!("        _ => None");
    println!("    }}");
    println!("}}\n");
}

fn print_regions_for_country_code(regions: BTreeMap<u16, Vec<String>>) {
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
struct Metadata {
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

    descriptors: Descriptors,
}

struct MetdataRegex {
    pub international_prefix: Option<CachedRegex>,
    pub national_prefix_for_parsing: Option<CachedRegex>,
    pub leading_digits: Option<CachedRegex>,
}

/// Descriptors for various types of phone number.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Descriptors {
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
struct Descriptor {
    national_number: String,

    possible_length: Vec<u16>,
    possible_local_length: Vec<u16>,

    example: Option<String>,
}

/// Description of a phone number format.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Format {
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
