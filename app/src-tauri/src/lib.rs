use serde::Serialize;
use std::process::Command;
use std::sync::Mutex;
use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use sysinfo::System;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
const IV_LENGTH: usize = 16;

/// Shared state to hold the device encryption key, fetched from the API after login.
/// Not hardcoded — rotatable without rebuilding the binary.
pub struct DeviceKeyState(pub Mutex<Option<String>>);

#[derive(Serialize, Clone, Debug)]
pub struct HardwareDevice {
    pub name: String,
    pub connected: bool,
    pub is_default: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct HardwareResponse {
    pub printers: Vec<HardwareDevice>,
    pub scanners: Vec<HardwareDevice>,
}

/// Collects hardware fingerprint data strictly — throws a descriptive error
/// if any critical piece of hardware information is unavailable.
fn get_hardware_info_strict() -> Result<String, String> {
    let mac = mac_address::get_mac_address()
        .map_err(|e| format!("Impossible de lire l'adresse MAC: {}", e))?
        .ok_or_else(|| "Aucune interface réseau détectée. Vérifiez vos adaptateurs réseau.".to_string())?;

    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_brand = sys.cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .ok_or_else(|| "Impossible de détecter le processeur de cet appareil.".to_string())?;

    if cpu_brand.trim().is_empty() {
        return Err("Le modèle du processeur est vide. L'appareil ne peut pas être identifié.".to_string());
    }

    let os_ver = System::os_version()
        .ok_or_else(|| "Version du système d'exploitation introuvable. L'appareil ne peut pas être identifié.".to_string())?;

    Ok(format!("MAC@{}:CPU_ID@{}:OS@{}", mac, cpu_brand, os_ver))
}

/// Stores the device encryption key received from the API.
/// Must be called immediately after login before any fingerprint operations.
#[tauri::command]
fn set_device_key(key: String, state: tauri::State<DeviceKeyState>) -> Result<(), String> {
    if key.is_empty() {
        return Err("La clé de chiffrement de l'appareil ne peut pas être vide.".to_string());
    }
    let mut lock = state.0.lock()
        .map_err(|_| "Erreur interne : impossible d'accéder à l'état de la clé.".to_string())?;
    *lock = Some(key);
    Ok(())
}

/// Returns the encrypted AES-256-CBC device fingerprint using the API-provided key.
/// Errors if the device key has not been set yet (user must be logged in first).
#[tauri::command]
fn get_device_fingerprint(state: tauri::State<DeviceKeyState>) -> Result<String, String> {
    let raw_data = get_hardware_info_strict()?;

    let lock = state.0.lock()
        .map_err(|_| "Erreur interne : impossible de lire la clé de chiffrement.".to_string())?;

    let key_str = lock.as_deref()
        .ok_or_else(|| "La clé d'empreinte n'est pas encore disponible. Veuillez vous connecter d'abord.".to_string())?;

    let mut key_bytes = [0u8; 32];
    let secret_bytes = key_str.as_bytes();
    let len = secret_bytes.len().min(32);
    key_bytes[..len].copy_from_slice(&secret_bytes[..len]);

    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut iv);

    let encryptor = Aes256CbcEnc::new(&key_bytes.into(), &iv.into());
    let encrypted = encryptor.encrypt_padded_vec_mut::<Pkcs7>(raw_data.as_bytes());

    let mut combined = iv.to_vec();
    combined.extend_from_slice(&encrypted);

    Ok(BASE64_STANDARD.encode(&combined))
}

#[derive(Serialize, Clone, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub fingerprint: String,
}

/// Returns combined device name + fingerprint.
/// Also fails strictly if hardware info or key is unavailable.
#[tauri::command]
fn get_device_info(state: tauri::State<DeviceKeyState>) -> Result<DeviceInfo, String> {
    let fingerprint = get_device_fingerprint(state)?;
    let name = System::host_name()
        .ok_or_else(|| "Nom d'hôte introuvable. L'appareil ne peut pas être identifié.".to_string())?;
    Ok(DeviceInfo { name, fingerprint })
}

#[tauri::command]
fn get_hardware_devices() -> HardwareResponse {
    let mut printers = Vec::new();
    let mut scanners = Vec::new();

    // --- 1. DETECT PRINTERS ---
    let mut detected_cups_default = None;
    
    // First find default printer via `lpstat -d`
    if let Ok(output) = Command::new("lpstat").arg("-d").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(target) = stdout.trim().split("destination: ").nth(1) {
                detected_cups_default = Some(target.trim().to_string());
            }
        }
    }

    if let Ok(output) = Command::new("lpstat").arg("-e").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().to_string();
                if !name.is_empty() {
                    let is_default = detected_cups_default.as_deref() == Some(&name);
                    printers.push(HardwareDevice {
                        name,
                        connected: true,
                        is_default,
                    });
                }
            }
        }
    } else if let Ok(output) = Command::new("powershell")
        .args(&["-Command", "Get-Printer | Select-Object Name, IsDefault | ConvertTo-Json"])
        .output() {
        // Windows fallback
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            #[derive(serde::Deserialize)]
            #[allow(non_snake_case)]
            struct WinPrinter {
                Name: String,
                IsDefault: Option<bool>,
            }

            if let Ok(list) = serde_json::from_str::<Vec<WinPrinter>>(&stdout) {
                for p in list {
                    printers.push(HardwareDevice {
                        name: p.Name,
                        connected: true,
                        is_default: p.IsDefault.unwrap_or(false),
                    });
                }
            } else if let Ok(p) = serde_json::from_str::<WinPrinter>(&stdout) {
                printers.push(HardwareDevice {
                    name: p.Name,
                    connected: true,
                    is_default: p.IsDefault.unwrap_or(false),
                });
            }
        }
    }

    // Always propose standard virtual PDF printer
    let has_pdf = printers.iter().any(|p| p.name.to_lowercase().contains("pdf"));
    if !has_pdf {
        let pdf_name = if cfg!(target_os = "windows") {
            "Microsoft Print to PDF"
        } else if cfg!(target_os = "macos") {
            "Enregistrer au format PDF"
        } else {
            "Virtual CUPS-PDF Printer"
        };
        
        printers.push(HardwareDevice {
            name: pdf_name.to_string(),
            connected: true,
            is_default: printers.is_empty(),
        });
    }

    // --- 2. DETECT SCANNERS ---
    if cfg!(target_os = "macos") {
        if let Ok(output) = Command::new("system_profiler").arg("SPUSBDataType").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut current_device_name = String::new();
                
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if trimmed.ends_with(':') {
                        current_device_name = trimmed.trim_end_matches(':').trim().to_string();
                    } else if trimmed.contains("Product ID:") || trimmed.contains("Vendor ID:") {
                        let lower_name = current_device_name.to_lowercase();
                        if lower_name.contains("scanner") 
                            || lower_name.contains("barcode") 
                            || lower_name.contains("reader") 
                            || lower_name.contains("symbol") 
                            || lower_name.contains("honeywell") 
                            || lower_name.contains("datalogic")
                            || lower_name.contains("zebra")
                        {
                            if !scanners.iter().any(|s: &HardwareDevice| s.name == current_device_name) {
                                scanners.push(HardwareDevice {
                                    name: current_device_name.clone(),
                                    connected: true,
                                    is_default: scanners.is_empty(),
                                });
                            }
                        }
                    }
                }
            }
        }
    } else if cfg!(target_os = "linux") {
        if let Ok(output) = Command::new("lsusb").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split("ID ").collect();
                    if parts.len() > 1 {
                        let desc = parts[1][9..].trim().to_string();
                        let lower = desc.to_lowercase();
                        if lower.contains("scanner") || lower.contains("barcode") || lower.contains("reader") {
                            scanners.push(HardwareDevice {
                                name: desc,
                                connected: true,
                                is_default: scanners.is_empty(),
                            });
                        }
                    }
                }
            }
        }
    } else {
        // Windows PowerShell PNP device search
        if let Ok(output) = Command::new("powershell")
            .args(&["-Command", "Get-PnpDevice -Class 'HIDClass','Keyboard' | Select-Object FriendlyName | ConvertTo-Json"])
            .output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                #[derive(serde::Deserialize)]
                #[allow(non_snake_case)]
                struct WinPnp {
                    FriendlyName: String,
                }

                if let Ok(list) = serde_json::from_str::<Vec<WinPnp>>(&stdout) {
                    for dev in list {
                        let lower = dev.FriendlyName.to_lowercase();
                        if lower.contains("scanner") || lower.contains("barcode") || lower.contains("reader") {
                            let is_default = scanners.is_empty();
                            scanners.push(HardwareDevice {
                                name: dev.FriendlyName,
                                connected: true,
                                is_default,
                            });
                        }
                    }
                } else if let Ok(dev) = serde_json::from_str::<WinPnp>(&stdout) {
                    let lower = dev.FriendlyName.to_lowercase();
                    if lower.contains("scanner") || lower.contains("barcode") || lower.contains("reader") {
                        let is_default = scanners.is_empty();
                        scanners.push(HardwareDevice {
                            name: dev.FriendlyName,
                            connected: true,
                            is_default,
                        });
                    }
                }
            }
        }
    }

    HardwareResponse {
        printers,
        scanners,
    }
}

#[tauri::command]
fn print_receipt(printer_name: String, content: String) -> Result<String, String> {
    let temp_path = std::env::temp_dir().join("aztea_receipt.txt");
    std::fs::write(&temp_path, &content)
        .map_err(|e| format!("Impossible d'écrire le fichier temporaire: {}", e))?;

    let result = if cfg!(target_os = "windows") {
        if printer_name.is_empty() {
            Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!("Get-Content '{}' | Out-Printer", temp_path.display()),
                ])
                .output()
        } else {
            Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!(
                        "Get-Content '{}' | Out-Printer '{}'",
                        temp_path.display(),
                        printer_name
                    ),
                ])
                .output()
        }
    } else {
        if printer_name.is_empty() {
            Command::new("lp")
                .arg(&temp_path)
                .output()
        } else {
            Command::new("lp")
                .arg("-d")
                .arg(&printer_name)
                .arg(&temp_path)
                .output()
        }
    };

    let _ = std::fs::remove_file(&temp_path);

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok("Impression envoyée avec succès".to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Impression échouée: {}", stderr))
            }
        }
        Err(e) => Err(format!("Erreur d'impression: {}", e)),
    }
}

fn user_downloads_dir() -> Result<std::path::PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let userprofile = std::env::var("USERPROFILE")
            .map_err(|_| "Variable USERPROFILE introuvable.".to_string())?;
        return Ok(std::path::PathBuf::from(userprofile).join("Downloads"));
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "Dossier personnel (HOME) introuvable.".to_string())?;
        return Ok(std::path::PathBuf::from(home).join("Downloads"));
    }
}

fn sanitize_pdf_filename(filename: &str) -> String {
    let mut name = filename.trim().to_string();
    if name.is_empty() {
        name = "aztea_document.pdf".to_string();
    }
    if !name.to_lowercase().ends_with(".pdf") {
        name.push_str(".pdf");
    }
    name.chars()
        .map(|c| {
            if "<>:\"/\\|?*".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect()
}

/// Enregistre un PDF dans le dossier Téléchargements de l'utilisateur (mode « Enregistrer au format PDF »).
#[tauri::command]
fn save_pdf_to_downloads(pdf_base64: String, filename: String) -> Result<String, String> {
    use base64::Engine;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(pdf_base64.trim())
        .map_err(|e| format!("PDF invalide (base64): {}", e))?;

    let downloads = user_downloads_dir()?;
    std::fs::create_dir_all(&downloads)
        .map_err(|e| format!("Impossible d'accéder à Téléchargements: {}", e))?;

    let safe_name = sanitize_pdf_filename(&filename);
    let path = downloads.join(&safe_name);
    std::fs::write(&path, &bytes)
        .map_err(|e| format!("Impossible d'écrire le fichier PDF: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn print_pdf_base64(printer_name: String, pdf_base64: String, filename: String) -> Result<String, String> {
    use base64::Engine;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(pdf_base64.trim())
        .map_err(|e| format!("PDF invalide (base64): {}", e))?;

    let safe_name = if filename.trim().is_empty() {
        "aztea_document.pdf".to_string()
    } else {
        filename
    };
    let temp_path = std::env::temp_dir().join(safe_name);
    std::fs::write(&temp_path, &bytes)
        .map_err(|e| format!("Impossible d'écrire le PDF temporaire: {}", e))?;

    let result = if cfg!(target_os = "windows") {
        if printer_name.is_empty() {
            Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!(
                        "Start-Process -FilePath '{}' -Verb Print",
                        temp_path.display()
                    ),
                ])
                .output()
        } else {
            Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!(
                        "Start-Process -FilePath '{}' -ArgumentList '/t','{}' -Verb Print",
                        temp_path.display(),
                        printer_name.replace('\'', "''")
                    ),
                ])
                .output()
        }
    } else if printer_name.is_empty() {
        Command::new("lp").arg(&temp_path).output()
    } else {
        Command::new("lp")
            .arg("-d")
            .arg(&printer_name)
            .arg(&temp_path)
            .output()
    };

    let _ = std::fs::remove_file(&temp_path);

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok("Document PDF envoyé à l'imprimante".to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Impression PDF échouée: {}", stderr))
            }
        }
        Err(e) => Err(format!("Erreur impression PDF: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Register the device key state (starts empty, filled after login)
        .manage(DeviceKeyState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            set_device_key,
            get_hardware_devices,
            get_device_fingerprint,
            get_device_info,
            print_receipt,
            print_pdf_base64,
            save_pdf_to_downloads
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
