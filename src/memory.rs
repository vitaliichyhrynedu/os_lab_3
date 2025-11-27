use crate::{
    hardware::mmu::Mmu,
    paging::{PageReplacementPolicy, Pfn, Vpn},
    process::{Pid, ProcessManager},
};

pub const FRAME_COUNT: usize = 8192;

// A mock swap page number
const SPN: usize = 0xDEADBEEF;

pub struct MemoryManager<P: PageReplacementPolicy> {
    pub frame_table: FrameTable,
    pub mmu: Mmu,
    policy: P,
    pub stats: MemoryStats,
}
impl<P: PageReplacementPolicy> MemoryManager<P> {
    pub fn new(policy: P) -> Self {
        Self {
            frame_table: FrameTable::new(),
            mmu: Mmu::new(),
            policy,
            stats: MemoryStats::new(),
        }
    }

    pub fn allocate_frame(&mut self, pm: &mut ProcessManager) -> Pfn {
        if let Some(pfn) = self.frame_table.get_unassigned() {
            return pfn;
        }
        let victim_pfn = self.policy.pick_victim(&mut self.frame_table, pm);
        self.evict_page(victim_pfn, pm);
        victim_pfn
    }

    pub fn evict_page(&mut self, pfn: Pfn, pm: &mut ProcessManager) {
        let (pid, vpn, spn) = {
            let fte = &mut self.frame_table.entries[pfn.0];
            (fte.pid, fte.vpn, fte.spn)
        };

        if let (Some(pid), Some(vpn)) = (pid, vpn) {
            let pte = pm.get_mut_pte(pid, vpn);
            pte.present = false;

            pte.pfn = match (pte.modified, spn) {
                (false, Some(spn)) => spn,
                _ => self.swap_out(),
            }
        }

        self.frame_table.entries[pfn.0].clear();
    }

    pub fn handle_page_fault(&mut self, pid: Pid, vpn: Vpn, pm: &mut ProcessManager) {
        let pte = pm.get_mut_pte(pid, vpn);
        let spn = if pte.pfn.0 != 0 { Some(pte.pfn) } else { None };
        let pfn = self.allocate_frame(pm);

        if let Some(spn) = spn {
            self.swap_in(spn);
        }

        let fte = &mut self.frame_table.entries[pfn.0];
        fte.assign(pid, vpn, spn);

        let pte = pm.get_mut_pte(pid, vpn);
        pte.pfn = pfn;
        pte.present = true;
        pte.referenced = true;
        pte.modified = false;
    }

    pub fn swap_in(&mut self, _spn: Pfn) {
        self.stats.swap_in_count += 1;
    }

    pub fn swap_out(&mut self) -> Pfn {
        self.stats.swap_out_count += 1;
        Pfn(SPN)
    }
}

pub struct MemoryStats {
    pub swap_out_count: u64,
    pub swap_in_count: u64,
}
impl MemoryStats {
    fn new() -> Self {
        Self {
            swap_out_count: 0,
            swap_in_count: 0,
        }
    }
}

pub struct FrameTable {
    pub entries: Vec<FrameTableEntry>,
}
impl FrameTable {
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(FRAME_COUNT);
        for idx in 0..FRAME_COUNT {
            entries.insert(idx, FrameTableEntry::new());
        }
        Self { entries }
    }

    pub fn get_unassigned(&self) -> Option<Pfn> {
        self.entries
            .iter()
            .position(|frame| frame.is_free())
            .map(|pfn| Pfn(pfn))
    }
}

pub struct FrameTableEntry {
    pub pid: Option<Pid>,
    pub vpn: Option<Vpn>,
    pub spn: Option<Pfn>,
}
impl FrameTableEntry {
    pub fn new() -> Self {
        FrameTableEntry {
            pid: None,
            vpn: None,
            spn: None,
        }
    }

    pub fn is_free(&self) -> bool {
        self.pid.is_none()
    }

    pub fn clear(&mut self) {
        self.pid = None;
        self.vpn = None;
        self.spn = None;
    }

    pub fn assign(&mut self, pid: Pid, vpn: Vpn, spn: Option<Pfn>) {
        self.pid = Some(pid);
        self.vpn = Some(vpn);
        self.spn = spn;
    }
}
