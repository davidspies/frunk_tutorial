pub struct HNil;

pub struct HCons<H, T> {
    pub head: H,
    pub tail: T,
}

// Helper function for constructing a cons cell
pub fn h_cons<H, T>(head: H, tail: T) -> HCons<H, T> {
    HCons { head, tail }
}
