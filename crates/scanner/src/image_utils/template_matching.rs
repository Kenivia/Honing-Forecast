use crate::scanner_state::{ScaledPosition, ScannerState};

impl ScannerState {
    pub fn images_close_enough(&self, a: ScaledPosition, b: ScaledPosition) -> bool {
        false
        //
    }
    pub fn template_match(
        &self,
        template: &[u8],
        bound: Option<ScaledPosition>,
    ) -> Option<ScaledPosition> {
        None
        //
    }
}
