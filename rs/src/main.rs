mod receiver;
use std::net::SocketAddr;

use axum::{Json, Router, routing::post};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::receiver::receive::Compiler;
use crate::receiver::receive::SubmitReq;

async fn submit(Json(req): Json<SubmitReq>) {
    let value = req.lang.clone();
    let compiler: Compiler = Compiler::new(req, value);
    let path = "../generate/test.cpp".to_string();

    // 用 spawn_blocking 把同步 IO 放到阻塞线程池
    let src_path = "../generate/test.cpp".to_string();
    let des_path = "../bin/test".to_string();

    match tokio::task::spawn_blocking(move || {
        // 1. 写文件
        compiler.write_code_to_file(&src_path)?;

        // 2. 编译
        compiler.compile_cpp(&src_path, &des_path)?;

        Ok::<_, std::io::Error>(())
    })
    .await
    {
        Ok(Ok(())) => println!("✅ 写文件 + 编译成功"),
        Ok(Err(e)) => eprintln!("❌ 写文件或编译失败：{}", e),
        Err(join_err) => eprintln!("❌ 阻塞任务 panic 或被取消：{}", join_err),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/submit", post(submit))
        .nest_service(
            "/",
            ServeDir::new("static").append_index_html_on_directories(true),
        )
        .layer(CorsLayer::permissive());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
