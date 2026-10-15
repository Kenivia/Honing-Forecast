use crate::{ocr::jobs::OCRLine, scanner_state::ScannerState, tooltip::hover::Hover};

const LOW_PRIORITY: usize = 100;

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
}
