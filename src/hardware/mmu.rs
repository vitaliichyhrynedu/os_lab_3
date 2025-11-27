use crate::paging::{PageTable, Pfn, Vpn};

pub struct Mmu;

impl Mmu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn translate(
        &self,
        page_table: &mut PageTable,
        vpn: Vpn,
        operation: Operation,
    ) -> TranslationResult {
        let pte = &mut page_table.entries[vpn.0];

        if !pte.present {
            return TranslationResult::PageFault;
        }

        pte.referenced = true;

        if let Operation::Write = operation {
            pte.modified = true;
        }

        TranslationResult::Success(pte.pfn)
    }
}

pub enum Operation {
    Read,
    Write,
}

pub enum TranslationResult {
    Success(Pfn),
    PageFault,
}
