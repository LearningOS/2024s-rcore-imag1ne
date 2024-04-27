#![allow(dead_code)]

/// [System Reset Extension (EID #0x53525354 "SRST")](https://github.com/riscv-non-isa/riscv-sbi-doc/blob/master/src/ext-sys-reset.adoc)
const SYSTEM_RESET_EID: usize = 0x53525354;
const SYSTEM_RESET_FID: usize = 0x0;

const SRST_RESET_TYPE_SHUTDOWN: u32 = 0x0;
const SRST_RESET_TYPE_COLD_REBOOT: u32 = 0x1;
const SRST_RESET_TYPE_WARM_REBOOT: u32 = 0x2;

const SRST_RESET_REASON_NO_REASON: u32 = 0x0;
const SRST_RESET_REASON_SYSTEM_FAILURE: u32 = 0x1;

const CONSOLE_PUTCHAR_EID: usize = 0x01;

const SET_TIMER_EID: usize = 0x54494D45;
const SET_TIMER_FID: usize = 0x0;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct SbiRet {
    pub error: usize,
    pub value: usize,
}

pub fn shutdown() -> SbiRet {
    let error;
    let value;

    unsafe {
        core::arch::asm!(
        "ecall",
        in("a6") SYSTEM_RESET_FID,
        in("a7") SYSTEM_RESET_EID,
        inlateout("a0") SRST_RESET_TYPE_SHUTDOWN as usize => error, // SRST System Reset Types
        inlateout("a1") SRST_RESET_REASON_NO_REASON as usize => value, // SRST System Reset Reason
        );
    }

    SbiRet { error, value }
}

pub fn console_putchar(ch: usize) -> usize {
    let error;

    unsafe {
        core::arch::asm!(
        "ecall",
        in("a7") CONSOLE_PUTCHAR_EID,
        inlateout("a0") ch => error,
        );
    }

    error
}

pub fn set_timer(timer: usize) -> SbiRet {
    let error;
    let value;

    unsafe {
        core::arch::asm!(
        "ecall",
        in("a6") SET_TIMER_FID,
        in("a7") SET_TIMER_EID,
        inlateout("a0") timer => error,
        lateout("a1") value,
        );
    }

    SbiRet { error, value }
}
