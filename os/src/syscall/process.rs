//! App management syscalls
use alloc::vec;
use alloc::vec::Vec;

use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{translated_byte_buffer, MapPermission, VPNRange, VirtAddr, VirtPageNum};
use crate::task::{
    change_program_brk, current_memory_set, current_task_start_time, current_task_status,
    current_task_syscall_count, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next, TaskStatus,
};
use crate::timer::{get_time_ms, get_time_us};

#[repr(C)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

impl TimeVal {
    pub fn into_bytes(self) -> Vec<u8> {
        into_bytes(self)
    }
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in its life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

impl TaskInfo {
    pub fn into_bytes(self) -> Vec<u8> {
        into_bytes(self)
    }
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("sys_ext");
}

pub fn sys_yield() -> isize {
    trace!("[kernel] sys_yield");
    suspend_current_and_run_next();
    0
}

fn into_bytes<T>(v: T) -> Vec<u8> {
    let v_size: usize = core::mem::size_of::<T>();
    let mut v_bytes = vec![0u8; v_size];
    let src = &v as *const _ as *const u8;
    unsafe {
        core::ptr::copy_nonoverlapping(src, v_bytes.as_mut_ptr(), v_size);
    }
    v_bytes
}

fn write_translated_byte_buffer(buffer: Vec<&mut [u8]>, bytes: &[u8]) {
    buffer.into_iter().fold(0, |start, buf| {
        let end = start + buf.len();
        buf.copy_from_slice(&bytes[start..end]);
        end
    });
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("[kernel] sys_get_time");
    let us = get_time_us();
    let tv = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let tv_bytes = tv.into_bytes();

    // write TimeVal
    let ts_bufs = translated_byte_buffer(
        current_user_token(),
        ts as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    write_translated_byte_buffer(ts_bufs, &tv_bytes);

    0
}

pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("[kernel] sys_task_info");
    let status = current_task_status();
    let syscall_times = current_task_syscall_count();
    let start_time = current_task_start_time();
    if start_time.is_none() {
        return -1;
    }
    let time = get_time_ms() - start_time.unwrap();
    let task_info = TaskInfo {
        status,
        syscall_times,
        time,
    };
    let ti_bytes = task_info.into_bytes();

    // write TaskInfo
    let ti_bufs = translated_byte_buffer(
        current_user_token(),
        ti as *const u8,
        core::mem::size_of::<TaskInfo>(),
    );
    write_translated_byte_buffer(ti_bufs, &ti_bytes);

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    let start_va = VirtAddr::from(start);
    if !start_va.aligned() {
        error!("{start_va:?} is not aligned.");
        return -1;
    }

    if port & !0x7 != 0 {
        error!("The remaining bits of {port} must be zero.");
        return -1;
    }

    if port & 0x7 == 0 {
        error!("Meaningless memory with {port}.");
        return -1;
    }

    let end_va = VirtAddr::from(start + len);
    let start_vpn = VirtPageNum::from(start_va);
    let end_vpn = end_va.ceil();
    let vpn_range = VPNRange::new(start_vpn, end_vpn);
    let mut memory_set = current_memory_set();
    for vpn in vpn_range {
        if let Some(pte) = memory_set.translate(vpn) {
            if pte.is_valid() {
                error!("{vpn:?} has been in use.");
                return -1;
            }
        }
    }

    let permission = MapPermission::from_bits((port as u8) << 1).unwrap() | MapPermission::U;
    memory_set.insert_framed_area(start_va, end_va, permission);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    let start_va = VirtAddr::from(start);
    if !start_va.aligned() {
        error!("{start_va:?} is not aligned.");
        return -1;
    }

    let start_vpn = VirtPageNum::from(start_va);
    let end_va = VirtAddr::from(start + len);
    let end_vpn = end_va.ceil();
    let vpn_range = VPNRange::new(start_vpn, end_vpn);
    let mut memory_set = current_memory_set();

    for vpn in vpn_range {
        if let Some(pte) = memory_set.translate(vpn) {
            if !pte.is_valid() {
                error!("trying to free an invalid {vpn:?}.");
                return -1;
            }
        }
    }

    memory_set.remove_framed_area(vpn_range);
    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
