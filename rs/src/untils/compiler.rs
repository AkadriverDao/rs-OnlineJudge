use std::{fs, io, path::Path, process::{Command, Stdio}};

use serde::Deserialize;


#[derive(Deserialize)]
pub struct SubmitReq {
    pub lang: String, // "cpp"
    pub code: String,
}

pub struct Compiler {
    submit: SubmitReq,
    compiler_lang: String,
}

impl Compiler {
    pub fn new(submit: SubmitReq, lang: String) -> Self {
        Self {
            submit: submit,
            compiler_lang: lang,
        }
    }

    pub fn write_code_to_file(&self, file_path: &str) -> io::Result<()> {
        let path = Path::new(file_path);

        // 1. 如果父目录不存在，递归创建
        if let Some(parent) = path.parent() {
            match fs::create_dir_all(parent) {
                Ok(_) => println!("✅ 目录已确认：{}", parent.display()),
                Err(e) => {
                    eprintln!("❌ 创建目录失败：{} -> {}", parent.display(), e);
                    return Err(e);
                }
            }
        }

        match fs::write(path, self.submit.code.clone()) {
            Ok(_) => {
                println!("✅ 已成功写入 {}", path.display());
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 写入文件失败：{} -> {}", path.display(), e);
                Err(e)
            }
        }
    }

    pub fn compile_cpp(&self, src_path: &String, out_bin: &String) -> std::io::Result<()> {
        // 1. 确保输出目录存在
        if let Some(dir) = std::path::Path::new(out_bin).parent() {
            std::fs::create_dir_all(dir)?;
        }

        // 2. 执行 g++
        let status = Command::new("g++")
            .arg(src_path) // 源文件
            .arg("-o") // 输出
            .arg(out_bin)
            .arg("-std=c++17") // 可按需要添加更多参数
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?; // 等待结束

        // 3. 检查返回码
        if status.success() {
            println!("✅ 编译成功 → {}", out_bin);
            Ok(())
        } else {
            eprintln!("❌ 编译失败，退出码：{:?}", status.code());
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "g++ 返回非零",
            ))
        }
    }
}
