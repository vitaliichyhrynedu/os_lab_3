use rand::Rng;

use crate::{
    hardware::mmu::Operation,
    paging::{PageTable, PageTableEntry, Vpn},
};

const WORKING_SET_HIT_RATE: f64 = 0.9;
const READ_RATE: f64 = 0.8;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(pub usize);

pub struct ProcessManager {
    processes: Vec<Process>,
}
impl ProcessManager {
    pub fn new() -> Self {
        Self { processes: vec![] }
    }

    pub fn spawn_process(&mut self, process: Process) {
        self.processes.push(process);
    }

    pub fn get_mut_process(&mut self, pid: Pid) -> Option<&mut Process> {
        self.processes.get_mut(pid.0)
    }

    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    pub fn get_mut_pte(&mut self, pid: Pid, vpn: Vpn) -> &mut PageTableEntry {
        &mut self.processes[pid.0].page_table.entries[vpn.0]
    }
}

pub struct Process {
    pub pid: Pid,
    pub page_table: PageTable,

    pub page_count: usize,
    pub working_set: WorkingSet,

    pub state: ProcessState,
}

impl Process {
    pub fn new(
        pid: Pid,

        page_count: usize,
        working_set_size: usize,

        lifespan: usize,
        working_set_lifespan: usize,
    ) -> Self {
        let page_table = PageTable::new(page_count);

        let mut working_set = WorkingSet::new(working_set_size);
        working_set.scramble(page_count);

        let state = ProcessState::new(lifespan, working_set_lifespan);

        Self {
            pid,
            page_table,

            page_count,
            working_set,

            state,
        }
    }

    pub fn request(&mut self) -> Request {
        if self.state.age >= self.state.lifespan {
            return Request::Termination;
        }

        if self.state.working_set_age >= self.state.working_set_lifespan {
            self.working_set.scramble(self.page_count);
            self.state.working_set_age = 0;
        }

        self.state.age += 1;
        self.state.working_set_age += 1;

        let mut rng = rand::rng();
        let working_set_hit = rng.random_bool(WORKING_SET_HIT_RATE);
        let vpn = if working_set_hit {
            let idx = rng.random_range(..self.working_set.size);
            self.working_set.vpns[idx]
        } else {
            rng.random_range(..self.page_count)
        };

        let is_read = rng.random_bool(READ_RATE);
        let operation = if is_read {
            Operation::Read
        } else {
            Operation::Write
        };

        Request::MemoryReference { vpn, operation }
    }
}

pub enum Request {
    MemoryReference { vpn: usize, operation: Operation },
    Termination,
}

pub struct ProcessState {
    pub age: usize,
    pub lifespan: usize,

    pub working_set_age: usize,
    pub working_set_lifespan: usize,
}

impl ProcessState {
    pub fn new(lifespan: usize, working_set_lifespan: usize) -> Self {
        Self {
            age: 0,
            lifespan,

            working_set_age: 0,
            working_set_lifespan,
        }
    }
}

pub struct WorkingSet {
    pub size: usize,
    pub vpns: Vec<usize>,
}

impl WorkingSet {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            vpns: Vec::with_capacity(size),
        }
    }

    pub fn scramble(&mut self, page_count: usize) {
        self.vpns.clear();
        let mut rng = rand::rng();
        for _ in 0..self.size {
            let vpn = rng.random_range(..page_count);
            self.vpns.push(vpn);
        }
    }
}
