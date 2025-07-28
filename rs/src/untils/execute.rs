use std::{ffi::{CStr, CString}, process::{Command, Stdio}};

use nix::{
    errno::Errno, fcntl::{open, OFlag}, sys::{stat::Mode, wait::waitpid}, unistd::{close, dup2, execvp, fork, ForkResult}
};

pub(crate) struct Execute {
    bin: String,
}

impl Execute {
    pub fn new(binary: String) -> Self {
        Self { bin: binary }
    }

    /// 运行 `self.bin`：
    /// - 如果 `input_file` 存在，把文件内容重定向到子进程 stdin；
    /// - 如果 `input_file` 为 None，子进程 stdin 为空。
    // pub fn fork_exec_maybe_input(&self, input_file: Option<&str>) -> nix::Result<()> {
    //     let c_prog = CString::new(self.bin.as_str())
    // .map_err(|_| Errno::EINVAL)?;
    //     // let c_path_ref: &CStr = &c_prog;

    //     // 打开输入文件（不存在则跳过）
    //     let fd_in = if let Some(file) = input_file {
    //         Some(open(
    //             &*CString::new(file).map_err(|_| Errno::EINVAL)?,
    //             OFlag::O_RDONLY,
    //             Mode::empty(),
    //         )?)
    //     } else {
    //         None
    //     };

    //     match unsafe { fork() }? {
    //         ForkResult::Parent { child } => {
    //             if let Some(fd) = fd_in {
    //                 close(fd)?;
    //             }
    //             waitpid(child, None)?;
    //             Ok(())
    //         }
    //         ForkResult::Child => {
    //             if let Some(fd) = fd_in {
    //                 dup2(fd, 0)?; // 重定向到 stdin
    //                 close(fd)?;
    //             }
    //             execvp(&c_prog, &[&c_prog])?;
    //             unreachable!();
    //         }

        // }
    // }

    pub fn fork_exec_maybe_input(&self, input_file: Option<&str>) -> Result<String, String> {
        let mut cmd = Command::new(&self.bin);
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // 可选输入重定向
        if let Some(file) = input_file {
            let file = std::fs::File::open(file).map_err(|e| e.to_string())?;
            cmd.stdin(Stdio::from(file));
        } else {
            cmd.stdin(Stdio::null());
        }

        let output = cmd.output().map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}