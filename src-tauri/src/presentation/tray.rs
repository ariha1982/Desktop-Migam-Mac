use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Manager,
};

use crate::infrastructure::SystemMetricsSnapshot;

const CPU_TRAY_ID: &str = "resource-cpu";
const MEMORY_TRAY_ID: &str = "resource-memory";
const ICON_SIZE: u32 = 64;

#[derive(Clone, Copy)]
enum ResourceKind {
    Cpu,
    Memory,
}

pub fn build(app: &App) -> tauri::Result<()> {
    let show_pet = MenuItem::with_id(app, "show_pet", "펫 표시", true, None::<&str>)?;
    let show_timer = MenuItem::with_id(app, "show_timer", "타이머 표시", true, None::<&str>)?;
    let show_gamcha = MenuItem::with_id(app, "show_gamcha", "GAMCHA!", true, None::<&str>)?;
    let start_focus = MenuItem::with_id(app, "start_focus", "집중 시작", true, None::<&str>)?;
    let pause_timer = MenuItem::with_id(app, "pause_timer", "일시정지", true, None::<&str>)?;
    let resume_timer = MenuItem::with_id(app, "resume_timer", "재개", true, None::<&str>)?;
    let stop_timer = MenuItem::with_id(app, "stop_timer", "타이머 중지", true, None::<&str>)?;
    let show_settings = MenuItem::with_id(app, "show_settings", "설정", true, None::<&str>)?;
    let emergency_stop = MenuItem::with_id(app, "emergency_stop", "긴급 중지", true, None::<&str>)?;
    let resume_pet = MenuItem::with_id(app, "resume_pet", "펫 다시 시작", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show_pet,
            &show_timer,
            &show_gamcha,
            &start_focus,
            &pause_timer,
            &resume_timer,
            &stop_timer,
            &show_settings,
            &emergency_stop,
            &resume_pet,
            &quit,
        ],
    )?;

    let cpu_icon = resource_icon(ResourceKind::Cpu, 0);
    TrayIconBuilder::with_id(CPU_TRAY_ID)
        .menu(&menu)
        .icon(cpu_icon)
        .tooltip("CPU 0% · 감자봇 시스템 모니터")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_pet" => show_window(app, "pet"),
            "show_timer" => {
                let _ = crate::presentation::commands::show_utility_window(
                    app.clone(),
                    "timer".to_owned(),
                );
            }
            "show_gamcha" => {
                let _ = crate::presentation::commands::show_utility_window(
                    app.clone(),
                    "gamcha".to_owned(),
                );
            }
            "show_settings" => {
                let _ = crate::presentation::commands::show_utility_window(
                    app.clone(),
                    "settings".to_owned(),
                );
            }
            "start_focus" => {
                let state = app.state::<crate::app_state::AppState>();
                if crate::presentation::commands::start_focus(app.clone(), state).is_ok() {
                    let _ = crate::presentation::commands::show_utility_window(
                        app.clone(),
                        "timer".to_owned(),
                    );
                }
            }
            "pause_timer" => {
                let state = app.state::<crate::app_state::AppState>();
                let _ = crate::presentation::commands::pause_timer(app.clone(), state);
            }
            "resume_timer" => {
                let state = app.state::<crate::app_state::AppState>();
                let _ = crate::presentation::commands::resume_timer(app.clone(), state);
            }
            "stop_timer" => {
                let state = app.state::<crate::app_state::AppState>();
                let _ = crate::presentation::commands::stop_timer(app.clone(), state);
            }
            "emergency_stop" => {
                let state = app.state::<crate::app_state::AppState>();
                let _ = crate::presentation::commands::emergency_stop(app.clone(), state);
            }
            "resume_pet" => {
                let state = app.state::<crate::app_state::AppState>();
                let _ = crate::presentation::commands::resume_pet(app.clone(), state);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    TrayIconBuilder::with_id(MEMORY_TRAY_ID)
        .menu(&menu)
        .icon(resource_icon(ResourceKind::Memory, 0))
        .tooltip("MEM 0% · 감자봇 시스템 모니터")
        .show_menu_on_left_click(true)
        .build(app)?;
    Ok(())
}

pub fn update_resource_indicators(app: &AppHandle, metrics: SystemMetricsSnapshot) {
    if let Some(tray) = app.tray_by_id(CPU_TRAY_ID) {
        let _ = tray.set_icon(Some(resource_icon(ResourceKind::Cpu, metrics.cpu_percent)));
        let _ = tray.set_tooltip(Some(format!(
            "CPU {}% · {}",
            metrics.cpu_percent,
            load_label(metrics.cpu_percent)
        )));
    }
    if let Some(tray) = app.tray_by_id(MEMORY_TRAY_ID) {
        let _ = tray.set_icon(Some(resource_icon(
            ResourceKind::Memory,
            metrics.memory_percent,
        )));
        let _ = tray.set_tooltip(Some(format!(
            "MEM {}% · {}",
            metrics.memory_percent,
            load_label(metrics.memory_percent)
        )));
    }
}

fn load_label(percent: u8) -> &'static str {
    match percent {
        0..=29 => "여유",
        30..=59 => "보통",
        60..=79 => "바쁨",
        _ => "과부하",
    }
}

fn resource_icon(kind: ResourceKind, percent: u8) -> Image<'static> {
    let accent = match kind {
        ResourceKind::Cpu => [0, 174, 239, 255],
        ResourceKind::Memory => [239, 39, 45, 255],
    };
    let mut canvas = IconCanvas::new();

    // Windows 트레이의 작은 16px 표시에서도 보이도록 얼굴이 캔버스를 거의 채운다.
    canvas.ellipse(32, 27, 29, 25, [15, 15, 15, 255]);
    canvas.ellipse(32, 27, 25, 21, [250, 250, 248, 255]);

    let (left_pupil_x, right_pupil_x, eye_y) = match percent {
        0..=29 => (23, 41, 24),
        30..=59 => (25, 43, 23),
        60..=79 => (27, 45, 26),
        _ => (28, 46, 28),
    };
    canvas.ellipse(21, 23, 10, 12, [18, 18, 18, 255]);
    canvas.ellipse(43, 23, 10, 12, [18, 18, 18, 255]);
    canvas.ellipse(21, 23, 7, 9, [255, 255, 255, 255]);
    canvas.ellipse(43, 23, 7, 9, [255, 255, 255, 255]);
    canvas.ellipse(left_pupil_x, eye_y, 3, 4, [18, 18, 18, 255]);
    canvas.ellipse(right_pupil_x, eye_y, 3, 4, [18, 18, 18, 255]);

    if percent >= 80 {
        canvas.line(32, 35, 32, 42, accent, 3);
        canvas.ellipse(32, 47, 3, 3, accent);
    } else if percent >= 60 {
        canvas.line(24, 42, 40, 42, [18, 18, 18, 255], 3);
    } else {
        canvas.ellipse(32, 42, 6, 5, [18, 18, 18, 255]);
        canvas.ellipse(32, 42, 3, 2, [250, 250, 248, 255]);
    }

    // 굵은 하단 막대는 색상과 현재 사용률을 한눈에 구분한다.
    canvas.rectangle(1, 51, 62, 12, [15, 15, 15, 255]);
    canvas.rectangle(5, 55, 54, 4, [205, 209, 211, 255]);
    let fill_width = i32::from(percent) * 54 / 100;
    if fill_width > 0 {
        canvas.rectangle(5, 55, fill_width, 4, accent);
    }

    Image::new_owned(canvas.pixels, ICON_SIZE, ICON_SIZE)
}

struct IconCanvas {
    pixels: Vec<u8>,
}

impl IconCanvas {
    fn new() -> Self {
        Self {
            pixels: vec![0; (ICON_SIZE * ICON_SIZE * 4) as usize],
        }
    }

    fn pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= ICON_SIZE as i32 || y >= ICON_SIZE as i32 {
            return;
        }
        let index = ((y as u32 * ICON_SIZE + x as u32) * 4) as usize;
        self.pixels[index..index + 4].copy_from_slice(&color);
    }

    fn rectangle(&mut self, x: i32, y: i32, width: i32, height: i32, color: [u8; 4]) {
        for pixel_y in y..(y + height) {
            for pixel_x in x..(x + width) {
                self.pixel(pixel_x, pixel_y, color);
            }
        }
    }

    fn ellipse(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius_x: i32,
        radius_y: i32,
        color: [u8; 4],
    ) {
        for y in (center_y - radius_y)..=(center_y + radius_y) {
            for x in (center_x - radius_x)..=(center_x + radius_x) {
                let dx = (x - center_x) as i64;
                let dy = (y - center_y) as i64;
                if dx * dx * i64::from(radius_y * radius_y)
                    + dy * dy * i64::from(radius_x * radius_x)
                    <= i64::from(radius_x * radius_x * radius_y * radius_y)
                {
                    self.pixel(x, y, color);
                }
            }
        }
    }

    fn line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: [u8; 4], width: i32) {
        let dx = (x1 - x0).abs();
        let step_x = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let step_y = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            for offset_y in -(width / 2)..=(width / 2) {
                for offset_x in -(width / 2)..=(width / 2) {
                    self.pixel(x0 + offset_x, y0 + offset_y, color);
                }
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let twice_error = error * 2;
            if twice_error >= dy {
                error += dy;
                x0 += step_x;
            }
            if twice_error <= dx {
                error += dx;
                y0 += step_y;
            }
        }
    }
}

fn show_window(app: &tauri::AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_icons_have_expected_rgba_dimensions() {
        let icon = resource_icon(ResourceKind::Cpu, 67);
        assert_eq!(icon.width(), ICON_SIZE);
        assert_eq!(icon.height(), ICON_SIZE);
        assert_eq!(icon.rgba().len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
        assert!(icon
            .rgba()
            .iter()
            .skip(3)
            .step_by(4)
            .any(|alpha| *alpha > 0));
    }

    #[test]
    fn load_labels_follow_four_usage_bands() {
        assert_eq!(load_label(29), "여유");
        assert_eq!(load_label(30), "보통");
        assert_eq!(load_label(60), "바쁨");
        assert_eq!(load_label(80), "과부하");
    }
}
