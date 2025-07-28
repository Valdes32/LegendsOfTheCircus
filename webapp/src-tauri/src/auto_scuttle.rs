use std::{sync::{Arc, Mutex}, time::Duration};
use tauri::{AppHandle, Manager};
use rdev::{simulate, EventType, Key, SimulateError};
use tokio::time;
use log::{info, error};

#[derive(Default)]
pub struct ScuttleState {
    pub target_server: Option<String>,
    pub running: bool,
    pub attempts: u32,
}

pub type SharedScuttle = Arc<Mutex<ScuttleState>>;

/// Команда запуска автоматического затопления
#[tauri::command]
pub async fn start_scuttle(
    state: tauri::State<'_, SharedScuttle>, 
    app: AppHandle,
    target_server: String
) -> Result<(), String> {
    {
        let mut s = state.lock().unwrap();
        if s.running { 
            return Err("already_running".into()); 
        }
        s.running = true;
        s.target_server = Some(target_server.clone());
        s.attempts = 0;
    }
    
    info!("Starting auto-scuttle for target server: {}", target_server);
    
    // Спавним фоновую задачу
    let state_clone = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        const MAX_ATTEMPTS: u32 = 10;
        
        loop {
            // ❶ Проверяем флаг завершения
            let should_continue = {
                let mut guard = state_clone.lock().unwrap();
                if !guard.running { 
                    info!("Auto-scuttle stopped by user");
                    return; 
                }
                guard.attempts += 1;
                let attempts = guard.attempts;
                if attempts > MAX_ATTEMPTS {
                    guard.running = false;
                    info!("Max attempts ({}) reached, stopping auto-scuttle", MAX_ATTEMPTS);
                    let _ = app.emit_all("scuttle-max-attempts", attempts);
                    return;
                }
                attempts
            };
            
            info!("Auto-scuttle attempt #{}", should_continue);
            
            // ❷ Эмулируем «стрелку влево» для вызова меню затопления
            match simulate(&EventType::KeyPress(Key::LeftArrow)) {
                Ok(()) => {
                    time::sleep(Duration::from_millis(100)).await;
                    let _ = simulate(&EventType::KeyRelease(Key::LeftArrow));
                    info!("Sent LeftArrow key, waiting 90 seconds for scuttle...");
                }
                Err(SimulateError) => {
                    error!("Failed to simulate LeftArrow key press");
                    continue;
                }
            }
            
            // ❸ Ждём 90 секунд при затоплении
            time::sleep(Duration::from_secs(90)).await;
            
            // ❹ Проверяем текущий сервер через существующую API BetterFleet
            match get_current_server(&app).await {
                Ok(current_server) => {
                    let target = state_clone.lock().unwrap().target_server.clone();
                    info!("Current server: {}, Target: {:?}", current_server, target);
                    
                    if let Some(target_server) = target {
                        if current_server == target_server {
                            info!("Successfully reached target server!");
                            state_clone.lock().unwrap().running = false;
                            let _ = app.emit_all("scuttle-success", &current_server);
                            return;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get current server: {}", e);
                }
            }
            
            // Небольшая пауза перед следующей попыткой
            time::sleep(Duration::from_secs(5)).await;
        }
    });
    
    Ok(())
}

/// Остановка процесса
#[tauri::command]
pub fn stop_scuttle(state: tauri::State<'_, SharedScuttle>) -> Result<(), String> {
    let mut guard = state.lock().unwrap();
    guard.running = false;
    guard.target_server = None;
    guard.attempts = 0;
    info!("Auto-scuttle stopped");
    Ok(())
}

/// Получение текущего статуса
#[tauri::command]
pub fn get_scuttle_status(state: tauri::State<'_, SharedScuttle>) -> Result<(bool, u32), String> {
    let guard = state.lock().unwrap();
    Ok((guard.running, guard.attempts))
}

/// Получаем текущий сервер через существующую систему BetterFleet
async fn get_current_server(app: &AppHandle) -> Result<String, String> {
    // Используем существующую команду get_server_ip из BetterFleet
    match app.try_state::<Arc<tokio::sync::RwLock<crate::api::Api>>>() {
        Some(api_state) => {
            let api_lock = api_state.read().await;
            // ✅ ИСПРАВЛЕНИЕ: Используем get_server_ip() вместо get_server_id()
            let server_ip = api_lock.get_server_ip().await;
            
            if server_ip.is_empty() {
                Err("No server detected".to_string())
            } else {
                // В BetterFleet server IP используется как идентификатор сервера
                Ok(server_ip)
            }
        }
        None => {
            // ✅ ИСПРАВЛЕНИЕ: Fallback без chrono, используем std::time
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let server_id = format!("server-{}", timestamp % 1000);
            Ok(server_id)
        }
    }
}
