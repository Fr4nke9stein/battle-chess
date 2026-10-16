//! Редактор баланса в браузере (с 0.38, просьба пользователя: сохранять одной кнопкой). Chrome и
//! Edge не дают странице открыть папку AppData, поэтому файл пишет сама игра: по кнопке «Править
//! баланс» она поднимает крошечный HTTP-сервер на 127.0.0.1 и открывает в браузере страницу
//! `tools/deck_builder.html` (вшита в exe). Страница берёт `balance.ini` по адресу `balance.ini`
//! (GET) и сохраняет его туда же (POST). Адрес содержит случайный ключ, а заголовки `Host` и
//! `Origin` сверяются — чужие сайты в браузере до сервера не дотянутся. Сервер поднимается при
//! запуске игры и живёт, пока она открыта; адрес постоянный (порт `PORT`, ключ — в
//! `%APPDATA%\BattleChess\editor.key`), поэтому открытая вкладка снова сохраняет, когда игру
//! перезапустили.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use eframe::egui;

use crate::balance;

const PAGE: &str = include_str!("../tools/deck_builder.html");
/// Постоянный порт (занят — берётся любой свободный, адрес тогда меняется от запуска к запуску).
const PORT: u16 = 47823;
/// Файл баланса — десятки килобайт; больше не принимаем.
const MAX_BODY: usize = 1 << 20;

/// Адрес страницы (`None` — сервер не поднялся).
static URL: OnceLock<Option<String>> = OnceLock::new();
/// Страница сохранила файл — меню перечитает баланс.
static SAVED: AtomicBool = AtomicBool::new(false);

/// Поднимает сервер редактора (при запуске игры).
pub fn start(ctx: &egui::Context) {
    URL.get_or_init(|| serve_forever(ctx.clone()));
}

/// Открывает редактор в браузере по умолчанию.
pub fn open(ctx: &egui::Context) -> Result<(), String> {
    let url = URL
        .get_or_init(|| serve_forever(ctx.clone()))
        .as_deref()
        .ok_or("не удалось запустить редактор баланса")?;
    let mut command = if cfg!(windows) {
        let mut command = std::process::Command::new("rundll32.exe");
        command.arg("url.dll,FileProtocolHandler");
        command
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
    } else {
        std::process::Command::new("xdg-open")
    };
    command
        .arg(url)
        .spawn()
        .map(drop)
        .map_err(|e| format!("не удалось открыть браузер: {e}"))
}

/// Сохраняла ли страница файл с прошлого вызова.
pub fn take_saved() -> bool {
    SAVED.swap(false, Ordering::Relaxed)
}

fn serve_forever(ctx: egui::Context) -> Option<String> {
    let listener = TcpListener::bind(("127.0.0.1", PORT))
        .or_else(|_| TcpListener::bind(("127.0.0.1", 0)))
        .ok()?;
    let host = format!("127.0.0.1:{}", listener.local_addr().ok()?.port());
    let key = key();
    let url = format!("http://{host}/{key}/");
    std::thread::Builder::new()
        .name("balance-editor".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = serve(stream, &host, &key, &ctx);
            }
        })
        .ok()?;
    Some(url)
}

/// Ключ адреса: из `editor.key` рядом с `balance.ini`, нет файла — новый случайный.
fn key() -> String {
    let file = balance::path().map(|p| p.with_file_name("editor.key"));
    if let Some(saved) = file.as_ref().and_then(|f| std::fs::read_to_string(f).ok()) {
        let saved = saved.trim();
        if saved.len() == 16 && saved.bytes().all(|b| b.is_ascii_hexdigit()) {
            return saved.to_owned();
        }
    }
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(std::process::id().into());
    let key = format!("{:016x}", hasher.finish());
    if let Some(file) = file {
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(file, &key);
    }
    key
}

/// Один запрос: страница, чтение или запись файла.
fn serve(stream: TcpStream, host: &str, key: &str, ctx: &egui::Context) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_owned();
    let target = parts.next().unwrap_or("").to_owned();
    let (mut length, mut host_ok, mut origin_ok) = (0usize, false, true);
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match name.trim().to_ascii_lowercase().as_str() {
            "content-length" => length = value.parse().unwrap_or(usize::MAX),
            "host" => host_ok = value == host,
            "origin" => origin_ok = value == format!("http://{host}"),
            _ => {}
        }
    }
    let path = target.split('?').next().unwrap_or("");
    let page = format!("/{key}/");
    let file = format!("/{key}/balance.ini");
    let mut out = stream;
    if !host_ok || !origin_ok {
        return reply(&mut out, "403 Forbidden", "text/plain", "чужой адрес");
    }
    match (method.as_str(), path) {
        ("GET", p) if p == page || p == format!("/{key}/index.html") => {
            reply(&mut out, "200 OK", "text/html", PAGE)
        }
        ("GET", p) if p == file => match balance::file_text() {
            Ok(text) => reply(&mut out, "200 OK", "text/plain", &text),
            Err(e) => reply(&mut out, "500 Internal Server Error", "text/plain", &e),
        },
        ("POST", p) if p == file => {
            if length > MAX_BODY {
                return reply(
                    &mut out,
                    "413 Payload Too Large",
                    "text/plain",
                    "файл слишком велик",
                );
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body)?;
            let Ok(text) = String::from_utf8(body) else {
                return reply(&mut out, "400 Bad Request", "text/plain", "файл не в UTF-8");
            };
            match balance::save_text(&text) {
                Ok(warnings) => {
                    SAVED.store(true, Ordering::Relaxed);
                    ctx.request_repaint();
                    reply(&mut out, "200 OK", "text/plain", &warnings.join("\n"))
                }
                Err(e) => reply(&mut out, "500 Internal Server Error", "text/plain", &e),
            }
        }
        _ => reply(
            &mut out,
            "404 Not Found",
            "text/plain",
            "нет такой страницы",
        ),
    }
}

fn reply(out: &mut TcpStream, status: &str, kind: &str, body: &str) -> std::io::Result<()> {
    write!(
        out,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}; charset=utf-8\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    out.write_all(body.as_bytes())?;
    out.flush()
}
