use rand::Rng;

use crate::{
    memory::{FRAME_COUNT, FrameTable},
    process::ProcessManager,
};

pub struct PageTable {
    pub entries: Vec<PageTableEntry>,
}
impl PageTable {
    pub fn new(page_count: usize) -> Self {
        let mut entries = Vec::with_capacity(page_count);
        for _ in 0..page_count {
            entries.push(PageTableEntry::new());
        }
        Self { entries }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vpn(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pfn(pub usize);

pub struct PageTableEntry {
    pub pfn: Pfn,
    pub present: bool,
    pub referenced: bool,
    pub modified: bool,
}
impl PageTableEntry {
    pub fn new() -> Self {
        Self {
            pfn: Pfn(0),
            present: false,
            referenced: false,
            modified: false,
        }
    }
}

pub trait PageReplacementPolicy {
    fn pick_victim(&mut self, frame_table: &mut FrameTable, pm: &mut ProcessManager) -> Pfn;
}

pub struct Random;
impl PageReplacementPolicy for Random {
    fn pick_victim(&mut self, _frame_table: &mut FrameTable, _pm: &mut ProcessManager) -> Pfn {
        let mut rng = rand::rng();
        Pfn(rng.random_range(..FRAME_COUNT))
    }
}

pub struct Clock {
    pub hand: usize,
}
impl Clock {
    pub fn new() -> Self {
        Self { hand: 0 }
    }

    pub fn inc(&mut self) {
        self.hand += 1;
        if self.hand == FRAME_COUNT {
            self.hand = 0;
        }
    }
}
impl PageReplacementPolicy for Clock {
    fn pick_victim(&mut self, frame_table: &mut FrameTable, pm: &mut ProcessManager) -> Pfn {
        loop {
            let fte = &mut frame_table.entries[self.hand];
            let pid = fte
                .pid
                .expect("pid must be Some, otherwise pick_victim wouldn't be called");
            let vpn = fte
                .vpn
                .expect("vpn must be Some, otherwise pick_victim wouldn't be called");
            let pte = pm.get_mut_pte(pid, vpn);
            if pte.referenced {
                pte.referenced = false;
                self.inc();
            } else {
                let victim = self.hand;
                self.inc();
                return Pfn(victim);
            }
        }
    }
}
