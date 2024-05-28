mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::MAX_SYSCALL_NUM;
use crate::loader::{get_app_data, get_num_app};

use crate::mm::MemorySet;
use crate::sync::UPSafeCell;
use crate::task::switch::__switch;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use alloc::vec::Vec;
pub use context::TaskContext;
use core::cell::{Ref, RefMut};
use lazy_static::lazy_static;
pub use task::{TaskControlBlock, TaskStatus};

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

lazy_static! {
    static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);

        let tasks: Vec<TaskControlBlock> = (0..num_app)
            .map(|app_id| TaskControlBlock::new(get_app_data(app_id), app_id))
            .collect();

        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            if inner.tasks[next].start_time.is_none() {
                inner.tasks[next].start_time = Some(get_time_ms());
            }
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);

            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("All applications completed!");
        }
    }

    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.borrow_mut();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        task0.start_time = Some(get_time_ms());
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();

        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn current_task_status(&self) -> TaskStatus {
        self.get_current_task_control_block_ref().task_status
    }

    fn current_task_syscall_add_count(&self, id: usize) {
        let mut tcb = self.get_current_task_control_block_mut();
        tcb.syscall_times[id] += 1;
    }

    fn current_task_syscall_count(&self) -> [u32; MAX_SYSCALL_NUM] {
        self.get_current_task_control_block_ref().syscall_times
    }

    fn current_task_start_time(&self) -> Option<usize> {
        self.get_current_task_control_block_ref().start_time
    }

    fn get_current_task_control_block_ref(&self) -> Ref<'_, TaskControlBlock> {
        let inner = self.inner.borrow();
        Ref::map(inner, |inner| &inner.tasks[inner.current_task])
    }

    fn get_current_task_control_block_mut(&self) -> RefMut<'_, TaskControlBlock> {
        let inner = self.inner.borrow_mut();
        RefMut::map(inner, |inner| &mut inner.tasks[inner.current_task])
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.borrow_mut();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.borrow_mut();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// Change the current 'Running' task's program break
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner.borrow_mut();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
    }

    fn get_current_memory_set(&self) -> RefMut<'_, MemorySet> {
        let tcb = self.get_current_task_control_block_mut();
        RefMut::map(tcb, |tcb| &mut tcb.memory_set)
    }
}

pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn current_task_status() -> TaskStatus {
    TASK_MANAGER.current_task_status()
}

pub fn current_task_syscall_add_count(id: usize) {
    TASK_MANAGER.current_task_syscall_add_count(id);
}

pub fn current_task_syscall_count() -> [u32; MAX_SYSCALL_NUM] {
    TASK_MANAGER.current_task_syscall_count()
}

pub fn current_task_start_time() -> Option<usize> {
    TASK_MANAGER.current_task_start_time()
}

#[allow(dead_code)]
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

/// Change the current 'Running' task's program break
pub fn change_program_brk(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_brk(size)
}

pub fn current_memory_set() -> RefMut<'static, MemorySet> {
    TASK_MANAGER.get_current_memory_set()
}
