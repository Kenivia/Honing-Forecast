// The last export's priors against the slot status rules. Icons are picked from the game tables.
use super::*;
use crate::{
    image_utils::common::Rect,
    tooltip::items::{ITEMS, VARIANTS},
};
use image::RgbaImage;

const AT: SlotAddress = SlotAddress {
    inventory_type: InventoryType::CharInventory,
    page_num: 0,
    pos_in_inv: (2, 3),
};

fn crop() -> OneIconConfig {
    OneIconConfig {
        data: RgbaImage::new(1, 1),
        name: String::new(),
        offset: Rect::ui((0.0, 0.0), 1.0, 1.0),
        tag: String::new(),
        required_confidence: None,
    }
}

fn slot(icon: &str, amount: &str) -> OneSlotInfo {
    OneSlotInfo {
        icon_name_score: Some((icon.into(), 0.9)),
        overlays: 0,
        alternatives: vec![],
        observed_number: crop(),
        observed_icon: crop(),
        display_icon: None,
        hovered: false,
        passed_over: (vec![], vec![]),
        tooltip_failed: false,
        amount: Some(amount.into()),
        amount_reads: vec![],
        tooltip_amount: None,
        tradability: None,
        label: None,
        amount_job: None,
        best_score: 0.9,
        raw_hash: 0,
    }
}

fn prior(icon: &str) -> Prior {
    Prior {
        address: AT,
        icon: icon.into(),
        item: Some(icon.into()),
        label: None,
        tradability: Some(Tradability::RosterBound),
        variants: vec![],
    }
}

// status, item, tradability, label, variants and assumed, as the page gets them
fn read(info: OneSlotInfo, prior: Option<Prior>) -> String {
    let mut state = ScannerState::default();
    state.set_priors(prior.into_iter().collect());
    state.slot_infos.insert(AT, info);
    let x = state.slot_result(&AT, &state.slot_infos[&AT], false);
    format!(
        "{:?} {:?} {:?} {:?} {:?} {}",
        x.status, x.item, x.tradability, x.label, x.variants, x.assumed
    )
}

// a material drawn with an icon of its own
fn material() -> String {
    ITEMS
        .iter()
        .filter_map(|x| x.icon.clone().filter(|_| x.label.is_some()))
        .find(|icon| !shares_icon(icon) && !is_chest_icon(icon))
        .unwrap()
}

// chests of one art that open differently, one of them `pick`
fn chest(pick: impl Fn(&Variant) -> bool) -> (String, u32) {
    VARIANTS
        .iter()
        .filter_map(|x| x.template())
        .find_map(|icon| {
            let info = slot(&icon, "3");
            let variants = slot_variants(&info);
            let one = variants.iter().find(|x| pick(x))?;
            (!open_alike(&variants)).then(|| (icon.clone(), one.id))
        })
        .unwrap()
}

#[test]
fn material_takes_the_prior_tradability() {
    let icon = material();
    assert_eq!(
        read(slot(&icon, "12"), None),
        format!("NeedTradability Some({icon:?}) None None [] false")
    );
    assert_eq!(
        read(slot(&icon, "12"), Some(prior(&icon))),
        format!("Good Some({icon:?}) Some(RosterBound) None [] true")
    );
}

#[test]
fn another_icon_is_no_match() {
    let icon = material();
    let other = Prior {
        icon: "not_this_one".into(),
        ..prior(&icon)
    };
    assert!(read(slot(&icon, "12"), Some(other)).starts_with("NeedTradability"));
}

#[test]
fn the_tooltip_wins() {
    let icon = material();
    let mut info = slot(&icon, "12");
    info.hovered = true;
    info.tradability = Some(Tradability::Tradable);
    assert_eq!(
        read(info, Some(prior(&icon))),
        format!("Good Some({icon:?}) Some(Tradable) None [] false")
    );
}

#[test]
fn a_full_stack_still_needs_its_tooltip() {
    let icon = material();
    assert!(read(slot(&icon, "9999+"), Some(prior(&icon))).starts_with("NeedHover"));
}

#[test]
fn a_shared_icon_takes_the_prior_label() {
    let item = ITEMS
        .iter()
        .find(|x| x.icon.as_deref().is_some_and(shares_icon))
        .unwrap();
    let icon = item.icon.clone().unwrap();
    assert!(read(slot(&icon, "5"), None).starts_with("NeedHover"));
    let label = item.label.clone();
    let with = Prior {
        label: label.clone(),
        ..prior(&icon)
    };
    assert_eq!(
        read(slot(&icon, "5"), Some(with)),
        format!("Good Some({icon:?}) Some(RosterBound) {label:?} [] true")
    );
}

#[test]
fn a_chest_takes_the_prior_variant() {
    let (icon, id) = chest(|x| !x.irrelevant);
    assert!(read(slot(&icon, "3"), None).starts_with("NeedHover"));
    let with = Prior {
        variants: vec![id],
        ..prior(&icon)
    };
    let got = read(slot(&icon, "3"), Some(with));
    assert!(got.starts_with("Good"), "{got}");
    assert!(got.ends_with(&format!("[{id}] true")), "{got}");
}

#[test]
fn a_chest_read_as_counting_for_nothing_is_irrelevant() {
    let (icon, id) = chest(|x| x.irrelevant);
    let with = Prior {
        variants: vec![id],
        ..prior(&icon)
    };
    let got = read(slot(&icon, "3"), Some(with));
    assert!(got.starts_with("Irrelevant"), "{got}");
    assert!(got.ends_with("true"), "{got}");
}

#[test]
fn a_hand_edit_carries_over() {
    let icon = material();
    let set = Prior {
        item: Some("something_else".into()),
        ..prior(&icon)
    };
    assert_eq!(
        read(slot(&icon, "12"), Some(set.clone())),
        "Good Some(\"something_else\") Some(RosterBound) None [] true"
    );
    let gone = Prior {
        item: None,
        ..prior(&icon)
    };
    assert_eq!(
        read(slot(&icon, "12"), Some(gone)),
        "Irrelevant None None None [] true"
    );
    // a tooltip since undoes it
    let mut info = slot(&icon, "12");
    info.hovered = true;
    info.tradability = Some(Tradability::CharBound);
    assert_eq!(
        read(info, Some(set)),
        format!("Good Some({icon:?}) Some(CharBound) None [] false")
    );
}
