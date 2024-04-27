//! App management syscalls
use crate::config::MAX_SYSCALL_NUM;
use crate::task::{
    current_task_start_time, current_task_status, current_task_syscall_count,
    exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
};
use crate::timer::{get_time_ms, get_time_us};

#[repr(C)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("[kernel] sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
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
    unsafe { *ti = task_info }

    0
}
