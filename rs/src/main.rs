use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio::task;
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;

mod untils; // 确保同目录下有 untils/mod.rs
mod question_service;
use crate::untils::{compiler::Compiler, execute::Execute,compiler::SubmitReq};
use crate::question_service::{init_question_service, save};

#[derive(Clone)]
pub enum TaskStatus {
    Running,
    Done(String),
}

type TaskMap = Arc<Mutex<HashMap<String, TaskStatus>>>;

// ------------------ /submit ------------------
pub async fn submit(
    State(state): State<TaskMap>,
    Json(req): Json<SubmitReq>,
) -> impl IntoResponse {
    let task_id = Uuid::new_v4().to_string();
    let task_id_clone = task_id.clone();
    let src_path = format!("../generate/{}.cpp", task_id);
    let des_path = format!("../bin/{}", task_id);
    let bin = des_path.clone();
    let lang = req.lang.clone();

    let state_clone = state.clone();
    state
        .lock()
        .unwrap()
        .insert(task_id.clone(), TaskStatus::Running);

    task::spawn_blocking(move || {
        let compiler = Compiler::new(req, lang);

        // 1. 写源码
        if let Err(e) = compiler.write_code_to_file(&src_path) {
            state_clone.lock().unwrap().insert(
                task_id.clone(),
                TaskStatus::Done(format!("写文件失败: {e}")),
            );
            return;
        }

        // 2. 编译
        if let Err(e) = compiler.compile_cpp(&src_path, &des_path) {
            state_clone
                .lock()
                .unwrap()
                .insert(task_id.clone(), TaskStatus::Done(format!("编译失败: {e}")));
            return;
        }

        // 3. 运行
        match Execute::new(bin).fork_exec_maybe_input(None) {
            Ok(out) => {
                state_clone
                    .lock()
                    .unwrap()
                    .insert(task_id.clone(), TaskStatus::Done(out));
            }
            Err(e) => {
                state_clone
                    .lock()
                    .unwrap()
                    .insert(task_id.clone(), TaskStatus::Done(format!("运行失败: {e}")));
            }
        }
    });

    task_id_clone
}

// ------------------ /result/:task_id ------------------
pub async fn result(
    State(state): State<TaskMap>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let map = state.lock().unwrap();
    match map.get(&task_id) {
        Some(TaskStatus::Running) => {
            axum::Json(serde_json::json!({ "status": "running" })).into_response()
        }
        Some(TaskStatus::Done(out)) => {
            axum::Json(serde_json::json!({ "status": "done", "result": out })).into_response()
        }
        None => (axum::http::StatusCode::NOT_FOUND, "task not found").into_response(),
    }
}

// ------------------ main ------------------
#[tokio::main]
async fn main() {
    let state: TaskMap = Arc::new(Mutex::new(HashMap::new()));
    let question_router = init_question_service().1;
    let app = Router::new()
        .route("/submit", post(submit))
        .route("/result/:task_id", get(result))
        .route("/save", post(save))
        .nest_service("/api", question_router) // 合并题目路由
        .nest_service(
            "/",
            ServeDir::new("static").append_index_html_on_directories(true),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}