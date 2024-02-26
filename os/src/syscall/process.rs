//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{translated_ptr, MapPermission}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_start_time, get_syscall_times, mmap, munmap, suspend_current_and_run_next, TaskStatus
    }, timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
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
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    // trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    // trace!("kernel: sys_get_time");
    let token = current_user_token();
    let time = get_time_us();
    unsafe {
        let sec_ptr = translated_ptr(token, (&mut (*ts).sec) as *mut usize);
        *sec_ptr = time / 1_000_000;
        let usec_ptr = translated_ptr(token, (&mut (*ts).usec) as *mut usize);
        *usec_ptr = time % 1_000_000;
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let token = current_user_token();
    let status = TaskStatus::Running;
    let syscall_times = get_syscall_times();
    let time = get_time_ms() - get_start_time();
    unsafe {
        let status_ptr = translated_ptr(token, (&mut (*ti).status) as *mut TaskStatus);
        *status_ptr = status;
        for i in 0..MAX_SYSCALL_NUM {
            let syscall_times_i_ptr = translated_ptr(token, (&mut (*ti).syscall_times[i]) as *mut u32);
            *syscall_times_i_ptr = syscall_times[i];
        }
        let time_ptr = translated_ptr(token, (&mut (*ti).time) as *mut usize);
        *time_ptr = time;
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    // Check start
    if start % PAGE_SIZE != 0 {
        error!("sys_mmap start address misaligned!");
        return -1;
    }
    let start_vpn = start / PAGE_SIZE;
    let num_pages = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    let end_vpn = start_vpn + num_pages;
    // Check port
    if port & !0x7 != 0 || port & 0x7 == 0 {
        error!("invalid port");
        return -1;
    }
    let perm = MapPermission::from_bits((port as u8) << 1 | MapPermission::U.bits()).unwrap();
    // Set U bit
    if mmap(start_vpn.into(), end_vpn.into(), perm) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let start_vpn = start / PAGE_SIZE;
    let num_pages = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    let end_vpn = start_vpn + num_pages;
    // Set U bit
    if munmap(start_vpn.into(), end_vpn.into()) {
        0
    } else {
        -1
    }
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
