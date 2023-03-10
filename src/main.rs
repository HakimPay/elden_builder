extern crate csv;
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate scraper;

use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};
use std::env;

static ELDEN_RING_WEAPONS_URL: &str = "https://eldenring.wiki.fextralife.com/Weapons+Comparison+Tables";
static ELDEN_RING_BASE_LINK: &str = "https://eldenring.wiki.fextralife.com/"; // weapon page url example = https://eldenring.wiki.fextralife.com/Academy+Glintstone+Staff

#[derive(Debug)]
struct Requirements {
    str: u8,
    dex: u8,
    int: u8,
    fai: u8,
    arc: u8,    
}

#[derive(Debug)]
struct Scalings {
    str: char,
    dex: char,
    int: char,
    fai: char,
    arc: char,
}

#[derive(Debug)]
struct Weapon {
    category: String,
    name: String,
    requirements: Requirements,
    scalings: Scalings,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    dbg!(&args);
    
    let mut filepath: String = String::from("");
    
    if args.len() >= 2 {
        filepath = String::from(&args[1]);
    }
    
    let mut weapons_database: Vec<Weapon> = Vec::new();

    // Not providing a filepath will read the default path
    if filepath.is_empty() {
        match csv::Writer::from_path("elden_builder.csv") {
            Ok(mut wrt) => {
                scrape_weapon_data(ELDEN_RING_WEAPONS_URL, &mut weapons_database);
                match wrt.write_record(&["Category", "Name", "Strength req", "Dexterity req", "Intelligence req", "Faith req", "Arcane req", "Strength scl", "Dexterity scl", "Intelligence scl", "Faith scl", "Arcane scl"]) {
                    Ok(_) => println!("Header written!"),
                    Err(_) => println!("Something went wrong when writing the header..."),
                }
                for weapon in &weapons_database {
                    match wrt.write_record(&[
                        &weapon.category, 
                        &weapon.name, 
                        &weapon.requirements.str.to_string(), &weapon.requirements.dex.to_string(), &weapon.requirements.int.to_string(), &weapon.requirements.fai.to_string(), &weapon.requirements.arc.to_string(),
                        &weapon.scalings.str.to_string(), &weapon.scalings.dex.to_string(), &weapon.scalings.int.to_string(), &weapon.scalings.fai.to_string(), &weapon.scalings.arc.to_string()
                        ]) {
                            Ok(_) => println!("{} has been written!", &weapon.name),
                            Err(_) => println!("Something went wrong when writing {}...", &weapon.name),
                        }
                }
                println!("Got {} weapons!", &weapons_database.len());
            },
            Err(_) => println!("Could not open csv file for writing..."),
        };
    } else {
        fill_weapons_database_from_csv(&filepath, &mut weapons_database);
        for weapon in weapons_database {
            println!("{:?}", weapon);
        }
    }
}

fn fill_weapons_database_from_csv(filepath: &String, weapons_database: &mut Vec<Weapon>) {
    match csv::Reader::from_path(filepath) {
        Ok(mut rdr) => {
            for weapon_record in rdr.records() {
                match weapon_record {
                    Ok(weapon_record) => { 
                        let mut weapon = Weapon {
                            category: String::from(""),
                            name: String::from(""),
                            requirements: Requirements {
                                str: 0,
                                dex: 0,
                                int: 0,
                                fai: 0,
                                arc: 0,
                            },
                            scalings: Scalings {
                                str: 'z',
                                dex: 'z',
                                int: 'z',
                                fai: 'z',
                                arc: 'z',
                            },
                        };
                        // Ugly stuff
                        weapon.category = weapon_record[0].to_string();
                        weapon.name = weapon_record[1].to_string();
                        weapon.requirements.str = weapon_record[2].parse().unwrap();
                        weapon.requirements.dex = weapon_record[3].parse().unwrap();
                        weapon.requirements.int = weapon_record[4].parse().unwrap();
                        weapon.requirements.fai = weapon_record[5].parse().unwrap();
                        weapon.requirements.arc = weapon_record[6].parse().unwrap();
                        weapon.scalings.str = weapon_record[7].parse().unwrap();
                        weapon.scalings.dex = weapon_record[8].parse().unwrap();
                        weapon.scalings.int = weapon_record[9].parse().unwrap();
                        weapon.scalings.fai = weapon_record[10].parse().unwrap();
                        weapon.scalings.arc = weapon_record[11].parse().unwrap();
                        weapons_database.push(weapon);
                     },
                    Err(_) => { println!("Could not read a record from {}", filepath) }
                }
            }
        },
        Err(_) => { println!("Could not open {}", filepath) }
    }
}

fn scrape_weapon_data(url: &str, weapons_database: &mut Vec<Weapon>) {
    let mut req = reqwest::get(url).unwrap();
    assert!(req.status().is_success());
    let doc_body = Html::parse_document(&req.text().unwrap());
    let table_body_selector = Selector::parse("tbody > tr > *:first-child > a").unwrap();
    for el in doc_body.select(&table_body_selector) {
        let weapon_name = el.inner_html();
        let link_ready_weapon_name = str::replace(weapon_name.as_str(), " ", "+");
        let weapon_link = format!("{}{}", ELDEN_RING_BASE_LINK, link_ready_weapon_name);
        scrape_weapon_page(&weapon_link, weapon_name.as_str(), weapons_database);
    }
}

fn scrape_weapon_page(url: &str, weapon_name: &str, weapons_database: &mut Vec<Weapon>) {
    let mut weapon_request = reqwest::get(url).unwrap();
    assert!(weapon_request.status().is_success());
    let weapon_page = Html::parse_document(&weapon_request.text().unwrap());

    let mut weapon: Weapon = Weapon {
        category: String::from("No type found"),
        name: String::from(weapon_name),
        requirements: Requirements {
            str: 0,
            dex: 0,
            int: 0,
            fai: 0,
            arc: 0,
        },
        scalings: Scalings {
            str: 'z',
            dex: 'z',
            int: 'z',
            fai: 'z',
            arc: 'z',
        },
    };
    
    // get the weapon's type
    let weapon_type_selector = Selector::parse(".wiki_table:nth-child(1) > tbody > :nth-child(5) > td:nth-child(1) > a").unwrap();
    for el in weapon_page.select(&weapon_type_selector) {
        weapon.category = el.inner_html();
    }

    lazy_static! {
        static ref RE_SKILL_NAME: Regex = Regex::new(r"[a-z]{3}").unwrap();
        static ref RE_SKILL_VALUE: Regex = Regex::new(r"\d{1,}").unwrap();
    }
    
    let weapon_skills_info = Selector::parse(".wiki_table:nth-child(1) > tbody > :nth-child(4) > td > .lineleft").unwrap();
    let mut select_iter = weapon_page.select(&weapon_skills_info);
    
    // weapon scaling
    let weapon_scalings = select_iter.next().unwrap().text().collect::<Vec<_>>();
    let mut weapon_scalings_str: String = String::from("");
    for str in weapon_scalings {
        weapon_scalings_str.push_str(str);
    }
    weapon_scalings_str = weapon_scalings_str.replace(" ", "").replace("\n", "").to_lowercase();
    // println!("{}", weapon_scalings_str.as_str()); //TODO: weapon scaling

    // weapon requirements
    let weapon_requirements = select_iter.next().unwrap().text().collect::<Vec<_>>();
    let mut weapon_requirements_str: String = String::from("");
    for str in weapon_requirements {
        weapon_requirements_str.push_str(str);
    }
    weapon_requirements_str = weapon_requirements_str.replace(" ", "").replace("\n", "").to_lowercase();
    let skill_names: Vec<String> = RE_SKILL_NAME.find_iter(&weapon_requirements_str).map(|val| val.as_str().to_string()).collect();
    let skill_values: Vec<u8> = RE_SKILL_VALUE.find_iter(&weapon_requirements_str).filter_map(|val| val.as_str().parse::<u8>().ok()).collect();
    let size = skill_names.len();
    for i in 0..size {
        let val = if i < skill_values.len() { skill_values[i] } else { 0 };
        match skill_names[i].as_str() {
            "str" => weapon.requirements.str = val,
            "dex" => weapon.requirements.dex = val,
            "int" => weapon.requirements.int = val,
            "fai" => weapon.requirements.fai = val,
            "arc" => weapon.requirements.arc = val,
            _ => { println!("{} is not a stat name", skill_names[i].as_str()); assert!(false); },
        }
    }

    if weapon.name == String::from("Steel-Wire Torch") {
        weapon.category = String::from("Torch"); // The weapon category for this specific torch is 2.5 for some reason so I just hardcoded it
    }

    weapons_database.push(weapon);
}