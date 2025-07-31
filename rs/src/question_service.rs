// src/question_service.rs
use axum::{
    extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, fs};
use std::sync::{Arc, Mutex};

use crate::untils::question::{Question, Templates};

type QuestionMap = Arc<Mutex<HashMap<String, Question>>>;

// 初始化一些示例题目
// fn init_questions() -> HashMap<String, Question> {
//     let mut questions = HashMap::new();

//     // 题目1: 两数之和
//     let mut q1 = Question::new(
//         "1".to_string(),
//         "两数之和".to_string(),
//         "给定一个整数数组 nums 和一个整数目标值 target，请你在该数组中找出 和为目标值 target  的那 两个 整数，并返回它们的数组下标。\n\n你可以假设每种输入只会对应一个答案。但是，数组中同一个元素在答案里不能重复出现。\n\n你可以按任意顺序返回答案。".to_string(),
//     );
    
//     let templates = Templates {
//         cpp: Some(r#"// 两数之和 - C++ 解法
// #include <vector>
// #include <unordered_map>

// using namespace std;

// class Solution {
// public:
//     vector<int> twoSum(vector<int>& nums, int target) {
//         unordered_map<int, int> hash;
//         for (int i = 0; i < nums.size(); ++i) {
//             int complement = target - nums[i];
//             if (hash.count(complement)) {
//                 return {hash[complement], i};
//             }
//             hash[nums[i]] = i;
//         }
//         return {};
//     }
// };"#.to_string())};
    
//     q1 = q1.with_templates(templates);
//     questions.insert("1".to_string(), q1);

//     // 题目2: 反转链表
//     let mut q2 = Question::new(
//         "2".to_string(),
//         "反转链表".to_string(),
//         "给你单链表的头节点 head ，请你反转链表，并返回反转后的链表。".to_string(),
//     );
    
//     let templates = Templates {
//         cpp: Some(r#"// 反转链表 - C++ 解法
// /**
//  * Definition for singly-linked list.
//  * struct ListNode {
//  *     int val;
//  *     ListNode *next;
//  *     ListNode() : val(0), next(nullptr) {}
//  *     ListNode(int x) : val(x), next(nullptr) {}
//  *     ListNode(int x, ListNode *next) : val(x), next(next) {}
//  * };
//  */
// class Solution {
// public:
//     ListNode* reverseList(ListNode* head) {
//         ListNode* prev = nullptr;
//         ListNode* curr = head;
//         while (curr) {
//             ListNode* next = curr->next;
//             curr->next = prev;
//             prev = curr;
//             curr = next;
//         }
//         return prev;
//     }
// };"#.to_string())};
    
//     q2 = q2.with_templates(templates);
//     questions.insert("2".to_string(), q2);

//     questions
// }

pub fn init_questions() -> HashMap<String, Question> {
    let mut map = HashMap::new();
    for entry in fs::read_dir("questions").expect("read question dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let json = fs::read_to_string(&path).expect("read json");
            let q: Question = serde_json::from_str(&json).expect("parse json");
            map.insert(q.id.clone(), q);
        }
    }
    map
}

// 获取所有题目列表
pub async fn get_questions(State(questions): State<QuestionMap>) -> impl IntoResponse {
    let questions = questions.lock().unwrap();
    let question_list: Vec<serde_json::Value> = questions
        .values()
        .map(|q| {
            json!({
                "id": q.id,
                "title": q.title,
                "description": q.description
            })
        })
        .collect();
    
    Json(question_list)
}

// // 获取单个题目详情
// pub async fn get_question(
//     State(questions): State<QuestionMap>,
//     Path(id): Path<String>,
// ) -> impl IntoResponse {
//     let questions = questions.lock().unwrap();
//     match questions.get(&id) {
//         Some(question) => Box::new(Json(question)),
//         None => Box::new((axum::http::StatusCode::NOT_FOUND, "Question not found").into_response()),
//     }
// }
pub async fn get_question(
    State(_questions): State<QuestionMap>,
    Path(id): Path<String>,
) -> Result<Json<Question>, (StatusCode, Json<serde_json::Value>)> {
    let questions = init_questions();
    match questions.get(&id) {
        Some(question) => Ok(Json(question.clone())), // Note: may need to clone
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Question not found"})),
        )),
    }
}
// 初始化题目服务
pub fn init_question_service() -> (QuestionMap, Router) {
    let questions = Arc::new(Mutex::new(init_questions()));
    
    let router = Router::new()
        .route("/questions", get(get_questions))
        .route("/questions/:id", get(get_question))
        .with_state(questions.clone());
    
    (questions, router)
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct QuestionInput {
//     id: String,
//     title: String,
//     description: String,
// }


/// 保存接口
pub async fn save(Json(payload): Json<Question>) -> &'static str {
    let filename = format!("questions/{}.json", payload.id);
    // 若目录不存在则创建
    let _ = fs::create_dir_all("questions");
    match fs::write(&filename, serde_json::to_string_pretty(&payload).unwrap()) {
        Ok(_) => "保存成功",
        Err(_) => "保存失败",
    }
}