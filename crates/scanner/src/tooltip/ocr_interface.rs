use std::mem::take;

use super::{
    chest::{ChestKind, ChestRow, chests_listing},
    items::{described_by, match_bar_title, match_title},
    title::join_title,
};
use crate::{ocr::jobs::OCRLine, tooltip::hover::Hover};
use crate::{scanner_state::ScannerState, tooltip::common::winner};

const LOW_PRIORITY: usize = 100;
pub const CENTRED_VOTE: usize = 100; // a title read with nothing over it outvotes any number of covered ones
pub const SETTLED: usize = 3; // agreeing reads after which the OCR stops for this hover

// rows this much like a chest's names settle it in one read
const EXACT_ROWS: f64 = 0.995;
// the text lines of one row of a chest
#[derive(Debug, Clone)]
pub struct ChestRowStrips {
    pub names: Vec<OCRLine>,
    pub count: OCRLine,
}
// the text lines of one frame of a hover, not sent off yet
#[derive(Debug, Default, Clone)]
pub struct Strips {
    // one per line, and whether every line was centred
    pub title: Option<(Vec<OCRLine>, bool)>,
    pub amount: Option<[OCRLine; 3]>,
    pub chest: Option<Vec<ChestRowStrips>>,
    // one per description line
    pub body: Option<Vec<OCRLine>>,
}

// The OCR jobs of one frame of a hover. Its votes are cast once every text is back.
#[derive(Debug, Default)]
pub struct PendingRead {
    pub hover: u32,
    pub group: u32,
    // one job per line, and whether every line was centred
    pub title: Option<(Vec<u32>, bool)>,
    pub amount: Option<[u32; 3]>,
    // per row: the name's lines, the count
    pub chest: Option<Vec<(Vec<u32>, u32)>>,
    pub body: Option<Vec<u32>>,
}

impl PendingRead {
    pub fn jobs(&self) -> impl Iterator<Item = u32> + '_ {
        let title = self.title.iter().flat_map(|x| x.0.iter().copied());
        let amount = self.amount.iter().flatten().copied();
        let chest = self.chest.iter().flatten();
        let chest = chest.flat_map(|row| row.0.iter().copied().chain([row.1]));
        title
            .chain(amount)
            .chain(chest)
            .chain(self.body.iter().flatten().copied())
    }
}
impl ScannerState {
    // sends lines off to be read, as one read of this hover
    pub fn send_read(&mut self, hover: &mut Hover, strips: Strips) {
        // a hover's first read goes ahead of other hovers' later ones, and one beside nothing we
        // know behind everything else
        let low = if hover.interest { 0 } else { LOW_PRIORITY };
        let priority = (hover.sent.into_iter().max().unwrap() + low).min(255) as u8;
        // each kind is a read of its own, so the short ones are not kept back by a chest's rows
        let id = hover.id;
        let mut reads = vec![];
        let next = |state: &mut ScannerState| {
            state.read_count += 1;
            PendingRead {
                hover: id,
                group: state.read_count,
                ..Default::default()
            }
        };
        if let Some((lines, centred)) = strips.title {
            let mut read = next(self);
            let jobs = lines
                .into_iter()
                .map(|line| self.request_ocr(line, priority, read.group))
                .collect();
            read.title = Some((jobs, centred));
            reads.push((0, read));
        }
        if let Some(images) = strips.amount {
            let mut read = next(self);
            read.amount = Some(images.map(|image| self.request_ocr(image, priority, read.group)));
            reads.push((1, read));
        }
        if let Some(rows) = strips.chest {
            let mut read = next(self);
            let mut jobs = vec![];
            for row in rows {
                let names = row
                    .names
                    .into_iter()
                    .map(|x| self.request_ocr(x, priority, read.group))
                    .collect();
                jobs.push((names, self.request_ocr(row.count, priority, read.group)));
            }
            read.chest = Some(jobs);
            reads.push((2, read));
        }
        if let Some(lines) = strips.body {
            let mut read = next(self);
            read.body = Some(
                lines
                    .into_iter()
                    .map(|line| self.request_ocr(line, priority, read.group))
                    .collect(),
            );
            reads.push((3, read));
        }
        for (kind, read) in reads {
            hover.waiting[kind] += 1;
            hover.sent[kind] += 1;
            self.pending_reads.push(read);
        }
    }

    // Casts the votes of every read whose texts are all back, then writes what its hover now says
    // onto the hovered slot and the chest list.
    pub fn resolve_reads(&mut self) {
        let mut hovers: Vec<Hover> = take(&mut self.past_hovers);
        let current = self.hover.take().map(|hover| {
            hovers.push(hover);
            hovers.len() - 1
        });
        let mut touched = vec![false; hovers.len()];
        if let Some(index) = current {
            touched[index] = hovers[index].missed == 0;
        }

        for read in take(&mut self.pending_reads) {
            let Some(index) = hovers.iter().position(|hover| hover.id == read.hover) else {
                continue;
            };
            if !read.jobs().all(|id| self.ocr_texts.contains_key(&id)) {
                self.pending_reads.push(read);
                continue;
            }
            let hover = &mut hovers[index];
            touched[index] = true;
            let text = |id: &u32| self.ocr_texts[id].clone();

            if let Some((lines, centred)) = &read.title {
                let read = join_title(&lines.iter().map(text).collect::<Vec<_>>());
                // a chest with a panel is told by its rows, a bar of gold only by its title
                let bar = || {
                    let bare = hover.chest_kind_votes.is_empty();
                    bare.then(|| match_bar_title(&read))
                        .flatten()
                        .map(str::to_string)
                };
                let title = match_title(&read)
                    .and_then(|item| item.title.clone())
                    .or_else(bar);
                if let Some(title) = title {
                    // a read with nothing over it that is a title to the letter settles it
                    let exact = *centred && read.eq_ignore_ascii_case(&title);
                    *hover.title_votes.entry(title).or_default() += if exact {
                        SETTLED * CENTRED_VOTE
                    } else if *centred {
                        CENTRED_VOTE
                    } else {
                        1
                    };
                }
                hover.last_read_title = read;
                hover.waiting[0] -= 1;
            }
            if let Some(jobs) = &read.amount {
                let [first, second, third] = jobs.each_ref().map(|id| {
                    text(id)
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>()
                });
                // the third is only there to break a tie
                let amount = if first == second {
                    first
                } else {
                    [first, second]
                        .into_iter()
                        .find(|x| *x == third)
                        .unwrap_or_default()
                };
                if !amount.is_empty() {
                    *hover.amount_votes.entry(amount).or_default() += 1;
                }
                hover.waiting[1] -= 1;
            }
            if let Some(rows) = read.chest {
                let mut texts = vec![];
                hover.chest_rows = rows
                    .into_iter()
                    .map(|(names, count)| {
                        let name_read = names.iter().map(text).collect::<Vec<_>>().join(" ");
                        texts.push((name_read.clone(), text(&count)));
                        ChestRow {
                            name_read,
                            count_read: text(&count),
                        }
                    })
                    .collect();
                // The chests that list these rows best get the vote. Several do when they list the
                // same things, or when a numeral that tells them apart did not read. Those that
                // open the way the header says go first: the header's wording varies ("Obtain one
                // of the following items at random" reads as obtain all, so that lets random ones
                // through too), so it only breaks ties.
                let (mut listing, whole) = chests_listing(&texts);
                let kind = winner(&hover.chest_kind_votes).map(|x| x.0);
                let opens = |x: ChestKind| {
                    Some(x) == kind
                        || (kind == Some(ChestKind::ObtainAll) && x == ChestKind::Random)
                };
                if listing.iter().any(|x| opens(x.0.kind)) {
                    listing.retain(|x| opens(x.0.kind));
                }
                let best = listing.iter().map(|x| x.1).fold(0.0, f64::max);
                // rows that read as a chest's to the letter, and are all of them, settle it
                let weight = if whole && best >= EXACT_ROWS {
                    SETTLED
                } else {
                    1
                };
                for (variant, _) in listing.iter().filter(|x| x.1 >= best - 1e-6) {
                    *hover.chest_votes.entry(variant.id).or_default() += weight;
                }
                hover.waiting[2] -= 1;
            }
            if let Some(lines) = &read.body {
                let lines: Vec<String> = lines.iter().map(text).collect();
                let title = winner(&hover.title_votes).map(|x| x.0);
                for (described, weight) in described_by(&lines, title.as_deref()) {
                    *hover.body_votes.entry(described).or_default() += weight;
                }
                hover.waiting[3] -= 1;
            }
        }

        for (index, mut hover) in hovers.into_iter().enumerate() {
            if touched[index] {
                self.write_hover(&mut hover, Some(index) != current);
            }
            if Some(index) == current {
                self.hover = Some(hover);
            } else {
                self.retire_hover(hover);
            }
        }
    }
}
