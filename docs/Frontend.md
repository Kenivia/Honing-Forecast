# Frontend

I will try my best to describe the general flow of information here. By no mean is this exhaustive and it does not differentiate between watch dependency and direct callbacks (although I'm pre sure everything here are direct callbacks).

This is very much subject to change and I will try my best to update when it does.

```mermaid
flowchart TB

subgraph UI

    StatusInput;
    MaterialDist;
    Instructions;

    RosterPage[Roster & Market];
end

StatusInput --> grids;
StatusInput --> tier;

Instructions --> |normal_progress, adv_progress| keyed_upgrades;

MaterialDist --> bound_budgets;
RosterPage --> roster_config

subgraph Stores[Pinia stores]

    roster_config
    subgraph active_profile;
        keyed_upgrades;
        grids --> keyed_upgrades;
        tier --> keyed_upgrades;
        bound_budgets;

    end

end


WASM[Rust via WASM]

keyed_upgrades --> build_payload
bound_budgets --> build_payload
roster_config --> build_payload



build_payload --> WASM

WASM --> worker_bundle[Various worker_bundles] --> |previous states aka instructions| build_payload


worker_bundle --> |avg cost, gold cost, states| UI
```
