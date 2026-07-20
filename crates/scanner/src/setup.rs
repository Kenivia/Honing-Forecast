use fast_image_resize::images::Image;
use serde::{Deserialize, Serialize};

use crate::scanner_state::{ScaledPosition, ScannerState};

#[derive(Debug, Serialize, Deserialize)]
pub struct OneIconSetup {
    data: Vec<u8>,
    name: String,
    width: usize,
    height: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomingNewIcon {
    position: ScaledPosition,
    name: String,
}
impl ScannerState {
    // pub fn access_icon_id(&self, id: usize) -> &[u8] {

    //     //
    // }

    pub fn setup(&mut self) {
        if self.incoming_new_icon.is_some() {
            let incoming: IncomingNewIcon = self.incoming_new_icon.clone().unwrap();
            let input: Vec<ScaledPosition> = vec![incoming.position];
            let data: Vec<u8> = self
                .downscale(&input)
                .into_iter()
                .flat_map(Image::into_vec)
                .collect();
            self.config.push(OneIconSetup {
                data,
                name: incoming.name,
                width: incoming.position.width(),
                height: incoming.position.height(),
            })
        }
    }
}
