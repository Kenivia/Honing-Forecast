// Sets the right answer for slots of a debug capture: the hand edits of its end record, which
// capture_replay --check holds a run against. The rest of the end record is what the page showed,
// which may be wrong, and is left alone.
//   cargo run --release --bin capture_edit -- <file.hfcap> list
//   cargo run --release --bin capture_edit -- <file.hfcap> set "Roster 0 5,1" amount=13 tradability=RosterBound [item=use_11_213@relic]
//   cargo run --release --bin capture_edit -- <file.hfcap> remove "Roster 0 5,1"
//   cargo run --release --bin capture_edit -- <file.hfcap> accept
// accept sets every slot to what the page showed, for a capture whose reading was all checked by
// hand; an irrelevant slot's edit has no item.
// A slot is named as the page's edits name it. The item defaults to the icon the page showed in
// that slot; tradability is Tradable, RosterBound or CharBound.
use hf_scanner::dev_utils::capture::{END, MAGIC};
use serde_json::{Value, json};
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (path, command) = (&args[0], args[1].as_str());
    let file = fs::read(path).unwrap();
    assert!(file.starts_with(MAGIC), "not an {MAGIC:?} capture");

    // where the end record starts, if there is one
    let mut at = MAGIC.len();
    let mut end_at = None;
    while at < file.len() {
        let length = u32::from_le_bytes(file[at + 1..at + 5].try_into().unwrap()) as usize;
        if file[at] == END {
            end_at = Some(at);
        }
        at += 5 + length;
    }
    let mut end: Value = match end_at {
        Some(at) => serde_json::from_slice(&file[at + 5..]).unwrap(),
        None => json!({}),
    };
    if !end["edits"].is_object() {
        end["edits"] = json!({});
    }

    match command {
        "list" => {
            for (key, edit) in end["edits"].as_object().unwrap() {
                println!(
                    "{key}: {} x{} {}",
                    edit["item"], edit["amount"], edit["tradability"]
                );
            }
            return;
        }
        "set" => {
            let key = &args[2];
            let mut parts = key.split(' ');
            let inventory_type = parts.next().unwrap();
            let page_num: u32 = parts.next().unwrap().parse().unwrap();
            let (row, column) = parts.next().unwrap().split_once(',').unwrap();
            let address = json!({
                "inventory_type": inventory_type,
                "page_num": page_num,
                "pos_in_inv": [row.parse::<u32>().unwrap(), column.parse::<u32>().unwrap()],
            });
            let shown = end["slots"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|x| x["address"] == address)
                .and_then(|x| x["icon_name_score"][0].as_str())
                .map(str::to_string);
            let mut edit = json!({
                "address": address,
                "item": shown,
                "amount": null,
                "tradability": null,
            });
            for field in &args[3..] {
                let (name, value) = field.split_once('=').unwrap();
                edit[name] = match name {
                    "amount" => json!(value.parse::<u32>().unwrap()),
                    "item" | "tradability" => json!(value),
                    _ => panic!("no field {name}"),
                };
            }
            assert!(
                edit["item"].is_string(),
                "no icon shown in {key}, give item="
            );
            println!("{key}: {edit}");
            end["edits"][key.as_str()] = edit;
        }
        // every slot as the page showed it, once that was checked by hand
        "accept" => {
            for slot in end["slots"].as_array().unwrap().clone() {
                let a = &slot["address"];
                let key = format!(
                    "{} {} {},{}",
                    a["inventory_type"].as_str().unwrap(),
                    a["page_num"],
                    a["pos_in_inv"][0],
                    a["pos_in_inv"][1]
                );
                let edit = match slot["status"].as_str().unwrap() {
                    "Irrelevant" => {
                        json!({"address": a, "item": null, "amount": null, "tradability": null})
                    }
                    "Good" => json!({
                        "address": a,
                        "item": if slot["label"].is_string() { &slot["label"] } else { &slot["icon_name_score"][0] },
                        "amount": slot["value"],
                        "tradability": slot["tradability"],
                    }),
                    status => panic!("{key} is {status}, resolve it first"),
                };
                end["edits"][key.as_str()] = edit;
            }
            println!("{} edits", end["edits"].as_object().unwrap().len());
        }
        "remove" => {
            let removed = end["edits"].as_object_mut().unwrap().remove(&args[2]);
            assert!(removed.is_some(), "no edit for {}", args[2]);
        }
        _ => panic!("list, set, remove or accept"),
    }

    let payload = serde_json::to_vec(&end).unwrap();
    let mut out = file[..end_at.unwrap_or(file.len())].to_vec();
    out.push(END);
    out.extend((payload.len() as u32).to_le_bytes());
    out.extend(payload);
    fs::write(path, out).unwrap();
}
