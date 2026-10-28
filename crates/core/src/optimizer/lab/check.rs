//! Whether a state is a plan the game allows: what the evaluator assumes and does not check.

use crate::advanced_honing::utils::MAX_ADV_STATE;
use crate::optimizer::response::Model;
use crate::state::get_pool_ids;
use crate::state_bundle::StateBundle;

/// why this state is not a plan the game allows, where it is not
pub fn invalid(state_bundle: &StateBundle) -> Option<String> {
    let juice_info = &state_bundle.prep_output.juice_info;
    let mut order = state_bundle.special_state.clone();
    order.sort();
    if order != (0..state_bundle.upgrade_arr.len()).collect::<Vec<_>>() {
        return Some("the free tap order is not a permutation".to_string());
    }
    let block = state_bundle.block();
    for (index, upgrade) in state_bundle.upgrade_arr.iter().enumerate() {
        let payload = &upgrade.state.payload;
        let (piece_type, level) = (upgrade.piece_type_usize, upgrade.upgrade_index);
        if !upgrade.is_normal_honing {
            if payload.len() != juice_info.adv_uindex_to_id[piece_type][level].len()
                || payload
                    .iter()
                    .any(|slot| slot.len() != 1 || slot[0] > MAX_ADV_STATE)
            {
                return Some(format!("upgrade {index} is not an advanced strategy"));
            }
            continue;
        }
        if payload.len() != upgrade.clean_prob_dist_len {
            return Some(format!(
                "upgrade {index} has {} slots, not {}",
                payload.len(),
                upgrade.clean_prob_dist_len
            ));
        }
        let pools = get_pool_ids(piece_type, level, juice_info);
        for (tap, slot) in payload.iter().enumerate() {
            for (at, id) in slot.iter().enumerate() {
                if slot[..at].contains(id) {
                    return Some(format!("upgrade {index} takes id {id} twice on tap {tap}"));
                }
                if !pools.iter().any(|pool| pool.contains(id)) {
                    return Some(format!("upgrade {index} cannot take id {id}"));
                }
            }
            if pools
                .iter()
                .any(|pool| slot.iter().filter(|id| pool.contains(id)).count() > 1)
            {
                return Some(format!(
                    "upgrade {index} takes two ids of one pool on tap {tap}"
                ));
            }
        }
        let model = Model::of(upgrade, juice_info, block);
        let indices = model.indices(payload);
        if model.canonical(&indices) != model.canonical(&model.filled(&indices)) {
            return Some(format!("upgrade {index} breaks the streak length {block}"));
        }
    }
    let chests = &state_bundle.prep_output.chests;
    if state_bundle.chest_states.len() != chests.len() {
        return Some("chest states are not one per chest".to_string());
    }
    for (key, chest) in chests {
        match state_bundle.chest_states.get(key) {
            Some(state) if chest.is_valid(state) => {}
            _ => return Some(format!("chest {key} is not opened in a way it can be")),
        }
    }
    None
}
