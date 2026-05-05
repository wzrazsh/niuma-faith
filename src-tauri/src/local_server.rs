use std::sync::Arc;
use std::thread;
use tiny_http::{Server, Response};
use crate::domain::task::TaskStatus;
use crate::tauri::state::AppState;

pub struct LocalHttpServer {
    app_state: Arc<AppState>,
    port: u16,
    token: String,
}

impl LocalHttpServer {
    pub fn new(app_state: Arc<AppState>, port: u16, token: String) -> Self {
        LocalHttpServer { app_state, port, token }
    }

    pub fn try_bind(port: u16) -> Result<Server, String> {
        Server::http(format!("127.0.0.1:{}", port)).map_err(|e| format!("Failed to bind HTTP server: {}", e))
    }

    pub fn run(self) {
        let server = match Self::try_bind(self.port) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("{}", e);
                return;
            }
        };
        tracing::info!("Local HTTP server listening on 127.0.0.1:{}", self.port);

        for request in server.incoming_requests() {
            let state = self.app_state.clone();
            let token = self.token.clone();
            thread::spawn(move || handle_request(request, state, &token));
        }
    }

    pub fn serve(server: Server, app_state: Arc<AppState>, token: String, port: u16) {
        tracing::info!("Local HTTP server listening on 127.0.0.1:{}", port);

        for request in server.incoming_requests() {
            let state = app_state.clone();
            let token = token.clone();
            thread::spawn(move || handle_request(request, state, &token));
        }
    }
}

fn handle_request(
    mut request: tiny_http::Request,
    state: Arc<AppState>,
    token: &str,
) {
    let url = request.url().to_string();
    if url != "/api/health" {
        let auth_header = request.headers().iter()
            .find(|h| h.field.as_str().to_ascii_lowercase() == "authorization")
            .map(|h| h.value.as_str().to_string());

        let expected = format!("Bearer {}", token);
        if auth_header != Some(expected) {
            let _ = request.respond(Response::from_string("Unauthorized").with_status_code(401));
            return;
        }
    }

    let method = request.method().as_str().to_string();
    let body = {
        let mut buf = String::new();
        if let Ok(_) = request.as_reader().read_to_string(&mut buf) { buf } else { String::new() }
    };

    let result = route_request(&method, &url, &body, &state);

    match result {
        Ok((status, body_str)) => {
            let response = Response::from_string(body_str).with_status_code(status);
            let _ = request.respond(response);
        }
        Err(e) => {
            let response = Response::from_string(format!("{{\"error\":\"{}\"}}", e)).with_status_code(400);
            let _ = request.respond(response);
        }
    }
}

fn route_request(method: &str, url: &str, body: &str, state: &Arc<AppState>) -> Result<(u16, String), String> {
    // GET /api/health
    if url == "/api/health" {
        return Ok((200, r#"{"status":"ok","version":"2.0.0"}"#.to_string()));
    }

    // GET /api/tasks/{session_id} — query task status
    if url.starts_with("/api/tasks/") && method == "GET" && !url.contains("/complete") && !url.contains("/abandon") {
        let session_id = url.trim_start_matches("/api/tasks/");
        let task = state.task.get_project_task(session_id)?;
        return Ok((200, serde_json::to_string(&task).unwrap_or_default()));
    }

    // POST /api/tasks/{session_id}/complete — complete task
    if url.ends_with("/complete") && method == "POST" {
        let session_id = url.trim_start_matches("/api/tasks/").trim_end_matches("/complete");
        let v: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
        let summary = v.get("summary").and_then(|s| s.as_str());
        let task = state.task.complete_project_task(session_id, summary)?;
        let resp = serde_json::json!({
            "task_id": task.id,
            "session_id": task.tool_session_id,
            "status": "completed",
            "duration_minutes": task.duration_seconds / 60,
            "faith_contributed": 0,
        });
        return Ok((200, resp.to_string()));
    }

    // POST /api/tasks/{session_id}/abandon — abandon task
    if url.ends_with("/abandon") && method == "POST" {
        let session_id = url.trim_start_matches("/api/tasks/").trim_end_matches("/abandon");
        let task = state.task.abandon_project_task(session_id)?;
        let resp = serde_json::json!({
            "task_id": task.id,
            "session_id": task.tool_session_id,
            "status": "abandoned",
        });
        return Ok((200, resp.to_string()));
    }

    // PUT /api/tasks/{session_id} — update task status
    if url.starts_with("/api/tasks/") && method == "PUT" {
        let session_id = url.trim_start_matches("/api/tasks/");
        let v: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
        let status = v["status"].as_str().ok_or("missing status")?;
        let task = state.task.update_project_task_status(session_id, status)?;
        let resp = serde_json::json!({
            "task_id": task.id,
            "session_id": task.tool_session_id,
            "status": format_status_for_api(&task.status),
        });
        return Ok((200, resp.to_string()));
    }

    // POST /api/tasks — create project task
    if url == "/api/tasks" && method == "POST" {
        let v: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
        let action = v["action"].as_str().unwrap_or("");

        match action {
            "create" => {
                let tool_name = v["tool_name"].as_str().unwrap_or("unknown");
                let session_id = v["session_id"].as_str().ok_or("missing session_id")?;
                let title = v["title"].as_str().unwrap_or("");
                let description = v["description"].as_str().unwrap_or("");

                match state.task.create_project_task("default_user", tool_name, session_id, title, description) {
                    Ok(task) => {
                        let resp = serde_json::json!({
                            "task_id": task.id,
                            "session_id": task.tool_session_id,
                            "status": "running",
                            "created_at": task.created_at,
                        });
                        return Ok((201, resp.to_string()));
                    }
                    Err(e) => {
                        if e.contains("session already exists") {
                            return Ok((409, format!("{{\"error\":\"{}\"}}", e)));
                        }
                        return Err(e);
                    }
                }
            }
            _ => return Err("unknown action".into())
        }
    }

    Err("not found".into())
}

fn format_status_for_api(status: &TaskStatus) -> &str {
    match status {
        TaskStatus::Running => "running",
        TaskStatus::Paused => "paused",
        TaskStatus::Completed => "completed",
        TaskStatus::Abandoned => "abandoned",
    }
}
