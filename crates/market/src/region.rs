// Switching region and character: game menu, Char. Select, Server, the region's tab and server, the character,
// Launch, until the Stronghold shows. Every step is checked by a reference crop. The server row and the
// character's slot are where their crops (server_<region>, char_<region>) were taken, so a machine's own
// characters need only their crops in its profile.
use crate::{
    game::{Game, ms},
    ui::{act, fail, log, looks, ref_at, ref_distance, reset, wait},
};

pub struct Region {
    pub name: &'static str,
    tab: (i32, i32),
    tab_ref: &'static str,
    server_ref: &'static str,
    char_ref: &'static str,
}

// this PC: NAE Thaemine, Kenicards (page 1, slot 2); EUC Arcturus, Kenimommy (page 1, slot 1)
pub const REGIONS: [Region; 2] = [
    Region {
        name: "NAE",
        tab: (874, 517),
        tab_ref: "tab_nae",
        server_ref: "server_nae",
        char_ref: "char_nae",
    },
    Region {
        name: "EUC",
        tab: (705, 517),
        tab_ref: "tab_euc",
        server_ref: "server_euc",
        char_ref: "char_euc",
    },
];

pub fn get(name: &str) -> &'static Region {
    REGIONS
        .iter()
        .find(|r| r.name.eq_ignore_ascii_case(name))
        .unwrap()
}

/// Logs the region's character in; true once it stands in the Stronghold. Picks up from character or
/// server select, so a failed try can simply run again.
pub fn switch(g: &mut Game, r: &Region) -> bool {
    log(&format!("switching to {}", r.name));
    let img = g.shot(None);
    if !looks(&img, "char_select") && !looks(&img, "server_select") {
        reset(g);
        if !act(
            g,
            3,
            0.0,
            3.0,
            |g| g.key("esc"),
            |img| looks(img, "game_menu"),
        ) || !act(
            g,
            3,
            0.0,
            3.0,
            |g| g.click(1204, 750),
            |img| looks(img, "charsel_confirm"),
        ) {
            fail(g, "the game menu did not lead to Char. Select");
            g.key("esc");
            return false;
        }
        g.click(906, 584);
        if wait(g, 90.0, |img| looks(img, "char_select")).is_none() {
            fail(g, "character select did not show");
            return false;
        }
    }
    if !looks(&g.shot(None), "server_select")
        && !act(
            g,
            3,
            0.0,
            20.0,
            |g| g.click(80, 1010),
            |img| looks(img, "server_select"),
        )
    {
        fail(g, "server select did not show");
        return false;
    }
    if !looks(&g.shot(None), r.tab_ref) {
        g.click(r.tab.0, r.tab.1);
        if wait(g, 5.0, |img| looks(img, "region_confirm")).is_none() {
            fail(g, "no Change Region Server prompt");
            return false;
        }
        g.click(918, 593);
        if wait(g, 30.0, |img| {
            looks(img, r.tab_ref) && !looks(img, "region_confirm")
        })
        .is_none()
        {
            fail(g, "the region tab did not change");
            return false;
        }
    }
    if !act(
        g,
        3,
        0.0,
        3.0,
        |g| g.click(960, ref_at(r.server_ref).1 + 15),
        |img| looks(img, r.server_ref),
    ) {
        fail(g, "the server was not picked");
        return false;
    }
    // Enter ignores clicks for a few seconds after the region changed (took at 3.5 s)
    if !act(
        g,
        3,
        3.0,
        10.0,
        |g| g.click(958, 923),
        |img| !looks(img, "server_select"),
    ) || wait(g, 90.0, |img| looks(img, "char_select")).is_none()
    {
        fail(g, "character select did not show");
        return false;
    }
    // roster page 1, then the character's slot (blue once picked)
    g.click(190, 1015);
    ms(800);
    let pick = |g: &mut Game| {
        g.click(ref_at(r.char_ref).0 + 108, 922);
        ms(300);
        g.move_to(960, 700);
    };
    if !act(g, 3, 0.0, 3.0, pick, |img| looks(img, r.char_ref)) {
        fail(g, "the character was not picked");
        return false;
    }
    g.click(852, 1012);
    g.move_to(1200, 600);
    // the attendance window (first log-in after 06:00 server time) covers the Stronghold's icons; Esc closes it
    let attendance = |img: &image::RgbaImage| ref_distance(img, "attendance", 0) < 5.0;
    if wait(g, 180.0, |img| {
        looks(img, "in_stronghold") || attendance(img)
    })
    .is_none()
    {
        fail(g, "the Stronghold did not show");
        return false;
    }
    // windows and the HUD come in after the scene
    ms(5000);
    if attendance(&g.shot(None)) {
        log("closing the attendance window");
        g.key("esc");
        if wait(g, 10.0, |img| looks(img, "in_stronghold")).is_none() {
            fail(g, "the attendance window did not close");
            return false;
        }
    }
    log(&format!("in {}", r.name));
    true
}
