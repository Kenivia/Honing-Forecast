// The Lost Ark Shop's Currency Exchange (F9, Currency Exchange, its Information page): the Royal Crystal
// sale offers, the Crystal purchase offers and the recent trades. Only Currency Exchange and Leave are pressed;
// the buying panel (bottom right) is neither read nor touched.
use crate::{
    game::{Game, ms},
    ui::*,
};
use image::RgbaImage;

#[derive(Debug, Default, serde::Serialize)]
pub struct Offer {
    /// gold per 100 crystals
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct Trade {
    /// gold per 100 crystals
    pub price: f64,
    pub gold: f64,
    pub crystals: f64,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct Exchange {
    /// when it was read (`upload::stamp`)
    pub at: String,
    /// "Royal Crystal Sale Offers: 5 sales with lowest price", as listed (dearest first)
    pub royal_sale: Vec<Offer>,
    /// "Crystal Purchase Offers: 5 highest price listings", as listed (dearest first)
    pub crystal_purchase: Vec<Offer>,
    /// "Recent Prices", as listed
    pub recent: Vec<Trade>,
}

/// Rows from the top until an empty one; a row read only in part is an error.
fn rows<T>(
    img: &RgbaImage,
    n: u32,
    cells: impl Fn(u32) -> Vec<(u32, u32, u32, u32)>,
    make: impl Fn(&[f64]) -> T,
) -> Result<Vec<T>, String> {
    let mut out = vec![];
    for k in 0..n {
        let reads: Vec<_> = cells(k)
            .into_iter()
            .map(|(x0, x1, y0, y1)| num(img, x0, x1, y0, y1, false))
            .collect();
        if reads
            .iter()
            .all(|r| r.as_ref().is_err_and(|e| e.is_empty()))
        {
            break;
        }
        let v: Vec<f64> = reads
            .into_iter()
            .map(|r| r.ok().flatten().ok_or(format!("row {k}")))
            .collect::<Result<_, _>>()?;
        out.push(make(&v));
    }
    Ok(out)
}

fn read(img: &RgbaImage) -> Result<Exchange, String> {
    let offer = |v: &[f64]| Offer {
        price: v[0],
        quantity: v[1],
    };
    let royal_sale = rows(
        img,
        5,
        |k| {
            vec![
                (400, 540, 310 + 54 * k, 333 + 54 * k),
                (140, 270, 310 + 54 * k, 333 + 54 * k),
            ]
        },
        offer,
    )?;
    if royal_sale.is_empty() {
        return Err("not loaded".into());
    }
    let crystal_purchase = rows(
        img,
        5,
        |k| {
            vec![
                (400, 540, 708 + 54 * k, 731 + 54 * k),
                (700, 830, 708 + 54 * k, 731 + 54 * k),
            ]
        },
        offer,
    )?;
    let recent = rows(
        img,
        7,
        |k| {
            let (y0, y1) = (303 + 39 * k, 322 + 39 * k);
            vec![
                (1000, 1200, y0, y1),
                (1290, 1480, y0, y1),
                (1600, 1760, y0, y1),
            ]
        },
        |v| Trade {
            price: v[0],
            gold: v[1],
            crystals: v[2],
        },
    )?;
    Ok(Exchange {
        at: crate::upload::stamp(),
        royal_sale,
        crystal_purchase,
        recent,
    })
}

/// The Information page, one unit of work.
pub fn scan(g: &mut Game) -> Option<Exchange> {
    crate::strips::context("currency exchange");
    unit(g, "currency exchange", read_page)
}

fn read_page(g: &mut Game) -> Option<Exchange> {
    close_market(g);
    g.key("f9");
    // the shop can take a while to load the first time
    if wait(g, 60.0, |img| looks(img, "shop_title")).is_none() {
        log("the shop did not open");
        return None;
    }
    // the button can ignore a click while the shop still loads
    let to_exchange = |g: &mut Game| {
        g.click(1636, 1036);
        ms(500);
        // off the panels: the bottom bar's currencies have tooltips that reach the tables
        g.move_to(1900, 600);
    };
    if !act(g, 3, 0.0, 7.0, to_exchange, |img| {
        looks(img, "exchange_title")
    }) {
        log("the currency exchange did not open");
        return None;
    }
    let info = |img: &RgbaImage| looks(img, "exchange_info");
    if !info(&g.shot(None)) {
        g.click(882, 40);
        g.move_to(1900, 600);
        if wait(g, 10.0, info).is_none() {
            log("the Information page did not open");
            return None;
        }
    }
    // the tables draw in for a moment after the title (a "500" read as "-...??0")
    ms(1500);
    let ex = steady(g, "currency exchange", read)?;
    close_shop(g);
    Some(ex)
}
