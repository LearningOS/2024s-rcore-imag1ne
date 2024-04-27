# Lab1 报告

## 总结

到目前为止，通过 ecall `sbi_set_timer` 设置时钟计数器触发时钟中断，进行抢占式调度，一次实现分时多任务系统。

这次实验中，通过记录任务首次被调度的时间，和维护每个任务的系统调用次数，增加一个获取当前任务信息的系统调用。

## 简答作业

1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test-2024S/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

   * QEMU emulator version 9.0.0
   * RustSBI version 0.4.0-alpha.1

   1. ch2b_bad_address.rs 向地址 0x0 写入数据，发生 Trap 异常 `Exception::StoreFault`(7), stval 寄存器给出了要写入的地址 0x0，spec 寄存器给出发生 Trap 前的最后一条指令地址为 0x804003ac。

      使用 rust-objdump 反汇编 ELF 文件：

      ` rust-objdump -S target/riscv64gc-unknown-none-elf/release/ch2b_bad_address`

      可以看到指令

      `804003ac: a3 00 00 00   sb      zero, 0x0(zero)`

      向地址 0x0 写入0。

      尝试改变 ch2b_bad_address.rs：

      ```rust
      // (0x0 as *mut u8).write_volatile(0);
      (0x3 as *mut u8).write_volatile(4);
      ```

      此时 spec 给出的地址为 0x804003ae，反汇编后可以看到：

      ```
      804003ac: 11 45         li      a0, 0x4
      804003ae: a3 01 a0 00   sb      a0, 0x3(zero)
      ```

      这两条指令向地址 0x3 写入 4。

   2. ch2b_bad_instruction.rs 和 ch2b_bad_register.rs 都发生了 Trap 异常 `Exception::IllegalInstraction`(2)。

2. 深入理解 [trap.S](https://github.com/LearningOS/rCore-Tutorial-Code-2024S/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

   1. L40：刚进入 `__restore` 时，`a0` 代表了什么值。请指出 `__restore` 的两种使用情景。

      `a0` 指向在 `__alltraps` 中分配 `TrapContext` 后的内核栈栈顶，第一种使用场景是在调用 `trap_handler` 后，特权级从 S 变回 U。第二种使用场景是启动运行一个应用。

   2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

      ```
      ld t0, 32*8(sp)
      ld t1, 33*8(sp)
      ld t2, 2*8(sp)
      csrw sstatus, t0
      csrw sepc, t1
      csrw sscratch, t2
      ```

      这几行汇编代码恢复了在内核栈中存储的 csr 状态。

      * `sstatus` 中的 SPP 给出 Trap 发生前，CPU 处于哪个特权级
      * `sepc` 记录了当 Trap 为异常时，Trap 发生之前执行的最后一条指令地址
      * `sscratch` 在这里被用来暂存 Trap 发生时，另一个特权级的栈顶置针。

   3. L50-L56：为何跳过了 `x2` 和 `x4`？

      ```
      ld x1, 1*8(sp)
      ld x3, 3*8(sp)
      .set n, 5
      .rept 27
         LOAD_GP %n
         .set n, n+1
      .endr
      ```

      `x2` 是 stack pointer，由于进入 `__alltraps` 后，本来指向用户栈的指针就被暂存在 `sscratch` 中，`x2` 中为指向内核栈的指针。所以要在后面使用 `csrr` 指令从 `sscratch` 中读取后再存入 `2*8(sp)` 中。

      `x4` 为 thread pointer，这里没有用到，因此不需要。

   4. L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

      ```
      csrrw sp, sscratch, sp
      ```

      `sp` 恢复为用户栈指针，`sscratch` 存入内核栈指针

   5. `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

      发生在 `sret` 。

      * 当 Trap 发生时：

         * `pc` 存入 `spec`，`pc` 被设置为 `stvec` 的值，也就是控制 Trap 处理代码的入口地址
         * `sstatus.sie` 会被保存在 `sstatus.spie` 字段中

        	* 同时 `sstatus.sie` 置零， 这也就在 Trap 处理的过程中屏蔽了所有 S 特权级的中断
        	* 之前的特权级模式被保存在 `sstatus.spp`，并把特权级设置为 S

      * 当 Trap 处理完毕 `sret` 的时候：
        * 从`spec` 恢复入 `pc`
        * 将 `sstatus.spie`  复制到 `sstatus.sie` 来恢复之前的中断启用状态
        * 特权级设置成 `sstatus.spp`
        * `sstatus.spie` 设置为 1，表示下次 S mode 中断被允许
        * `sstatus.spp` 清零

   6. L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

      ```
      csrrw sp, sscratch, sp
      ```

      `sp` 为指向内核栈的指针，`sscratch` 为指向用户栈的指针

   7. 从 U 态进入 S 态是哪一条指令发生的？

      当 CPU 执行完一条指令并准备从用户特权级 陷入（ `Trap` ）到 S 特权级的时候，CPU 会跳转到 `stvec` 所设置的 Trap 处理入口地址，并将当前特权级设置为 S ，然后从Trap 处理入口地址处开始执行。也就是进入 `__alltraps` 的时候就进入 S mode 了。

## **荣誉准则**

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > * *无*

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > * [RISC-V SBI specification](https://github.com/riscv-non-isa/riscv-sbi-doc)
   > * D. A. Patterson and A. Waterman, *The RISC-V reader: an open architecture atlas*, Book version, First edition: 1.0.0. Berkeley, California: Strawberry Canyon LLC, 7.
   > * A. Waterman, K. Asanovic, and C. Division, “Volume I: Unprivileged ISA”.
   > * A. Waterman, K. Asanovic, J. Hauser, and C. Division, “Volume II: Privileged Architecture”.

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。