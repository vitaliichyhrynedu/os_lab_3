use crate::{
    hardware::mmu::{Operation, TranslationResult},
    memory::MemoryManager,
    paging::{PageReplacementPolicy, Vpn},
    process::{Pid, ProcessManager},
};

pub struct Kernel<P: PageReplacementPolicy> {
    pub mm: MemoryManager<P>,
    pub pm: ProcessManager,
}

impl<P: PageReplacementPolicy> Kernel<P> {
    pub fn new(policy: P) -> Self {
        Self {
            mm: MemoryManager::new(policy),
            pm: ProcessManager::new(),
        }
    }

    pub fn access_memory(&mut self, pid: Pid, vpn: Vpn, operation: Operation) -> AccessResult {
        let page_table = &mut self
            .pm
            .get_mut_process(pid)
            .expect("Process not found")
            .page_table;
        let pfn = self.mm.mmu.translate(page_table, vpn, operation);
        match pfn {
            TranslationResult::Success(_) => AccessResult::Hit,
            TranslationResult::PageFault => {
                self.mm.handle_page_fault(pid, vpn, &mut self.pm);
                AccessResult::Miss
            }
        }
    }
}

pub enum AccessResult {
    Hit,
    Miss,
}
