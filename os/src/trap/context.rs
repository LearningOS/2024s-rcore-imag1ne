#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    /// init app context
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        // read sstatus
        let mut sstatus: usize;
        unsafe {
            core::arch::asm!(
            "csrrs {sstatus}, 0x100, x0",
            sstatus = out(reg) sstatus
            );
        }
        // set ssp as user
        sstatus &= !(1 << 8);

        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry, // entry point of app
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}