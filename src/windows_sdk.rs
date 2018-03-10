use std::fs::DirEntry;
use std::path::PathBuf;
use std::io;
use winreg::RegKey;
use winreg::enums::*;
use std::env;
use std::cmp::Ordering;

static MS_REG_PATH: &str = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots\";
static WOW_REG_PATH: &str = r"WOW6432Node\Microsoft\Windows Kits\Installed Roots\";
static REG_VALUE_10: &str = "KitsRoot10";
static REG_VALUE_81: &str = "KitsRoot81";
static PROGRAM_FILES_ENV: &str = "%ProgramFiles%";
static PROGRAM_FILES_X86_ENV: &str = "%ProgramFiles(x86)%";
static WIN_KITS_PATH_10: &str = r"Windows Kits\10";
static WIN_KITS_PATH_81: &str = r"Windows Kits\8.1";

fn get_windows_sdk_path_from_regsitry(reg_path: &str, reg_value: &str) -> io::Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let installed_roots_subkey = hklm.open_subkey(reg_path)?;
    let kit_root_path: String = installed_roots_subkey.get_value(reg_value)?;
    Ok(kit_root_path)
}

fn get_windows_sdk_path_from_regsitry_logged(reg_path: &str, reg_value: &str) -> Option<String> {
    match get_windows_sdk_path_from_regsitry(reg_path, reg_value) {
        Ok(s) => Some(s),
        Err(err) => {
            debug!(
                "Can't find registry key {} and value {} : {:?}",
                reg_path, reg_value, err
            );
            None
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum SdkVersion {
    Win8,
    Win10,
}

#[derive(PartialEq, Eq, Debug)]
pub struct WindowsSdk {
    pub windows_version: SdkVersion,
    pub windows_target_platform_version: String,
    pub path: PathBuf,
}

fn check_add_windows_10_sdk(
    path: &str,
    path_opt: Option<&str>,
    mut valid_instance: Vec<WindowsSdk>,
) -> Vec<WindowsSdk> {
    let mut path_buf = PathBuf::new();
    path_buf.push(path);
    if let Some(some_path) = path_opt {
        path_buf.push(some_path);
    }
    debug!("Check {:?}", path_buf);
    path_buf.push("Include");
    if !path_buf.exists() {
        debug!("{:?} Not found", path_buf);
        return valid_instance;
    }
    if let Ok(dir) = path_buf.read_dir() {
        let kit_paths: Vec<DirEntry> = dir.filter_map(|sub_res| sub_res.ok())
            .filter(|sub| {
                sub.file_name()
                    .to_str()
                    .map(|f| f.starts_with("10"))
                    .unwrap_or(false)
            })
            .collect();
        //TODO sort ?
        for sdk_dir in kit_paths {
            let mut windows_header = sdk_dir.path().clone();
            windows_header.push("um");
            windows_header.push("windows.h");
            if !windows_header.exists() {
                debug!("{:?} Not found ", windows_header);
                continue;
            }
            debug!("{:?} found ", windows_header);

            let mut ddk_header = sdk_dir.path().clone();
            ddk_header.push("shared");
            ddk_header.push("sdkddkver.h");
            if !ddk_header.exists() {
                debug!("{:?} Not found ", ddk_header);
                continue;
            }
            debug!("{:?} found ", ddk_header);
            if let Some(version) = sdk_dir.path().file_name() {
                if let Some(version_str) = version.to_str() {
                    debug!("Found {} in {:?}", version_str, sdk_dir);
                    valid_instance.push(WindowsSdk {
                        windows_version: SdkVersion::Win10,
                        windows_target_platform_version: version_str.to_string(),
                        path: sdk_dir.path(),
                    });
                } else {
                    error!("Can't convert {:?}", version);
                }
            } else {
                debug!("Can't get file name of {:?}", sdk_dir);
            }
        }
    } else {
        error!("Can't read dir {:?}", path_buf);
    }
    valid_instance
}

fn check_add_windows_81_sdk(
    path: &str,
    path_opt: Option<&str>,
    mut valid_instance: Vec<WindowsSdk>,
) -> Vec<WindowsSdk> {
    let mut path_buf = PathBuf::new();
    path_buf.push(path);
    if let Some(some_path) = path_opt {
        path_buf.push(some_path);
    }
    debug!("Check {:?}", path_buf);
    path_buf.push("Include");
    if path_buf.exists() {
        if path_buf.read_dir().is_ok() {
            //TODO sort ?

            valid_instance.push(WindowsSdk {
                windows_version: SdkVersion::Win8,
                windows_target_platform_version: "8.1".into(),
                path: path_buf,
            });
        }
    } else {
        debug!("Can't get file name of {:?}", path_buf);
    }
    valid_instance
}

pub fn get_windows_sdk() -> Vec<WindowsSdk> {
    let mut windows_sdk = Vec::<WindowsSdk>::new();

    debug!("Looking for Windows 10 SDK");
    if let Some(path) = get_windows_sdk_path_from_regsitry_logged(MS_REG_PATH, REG_VALUE_10) {
        windows_sdk = check_add_windows_10_sdk(&path, None, windows_sdk);
    }
    if let Some(path) = get_windows_sdk_path_from_regsitry_logged(WOW_REG_PATH, REG_VALUE_10) {
        windows_sdk = check_add_windows_10_sdk(&path, None, windows_sdk);
    }

    if let Some(program_files_path) =
        env::var_os(PROGRAM_FILES_ENV).and_then(|p| p.into_string().ok())
    {
        windows_sdk =
            check_add_windows_10_sdk(&program_files_path, Some(WIN_KITS_PATH_10), windows_sdk);
    }
    if let Some(program_files_x86_path) =
        env::var_os(PROGRAM_FILES_X86_ENV).and_then(|p| p.into_string().ok())
    {
        windows_sdk = check_add_windows_10_sdk(
            &program_files_x86_path,
            Some(WIN_KITS_PATH_10),
            windows_sdk,
        );
    }
    debug!("Looking for Windows 8.1 SDK");
    if let Some(path) = get_windows_sdk_path_from_regsitry_logged(MS_REG_PATH, REG_VALUE_81) {
        windows_sdk = check_add_windows_81_sdk(&path, None, windows_sdk);
    }
    if let Some(path) = get_windows_sdk_path_from_regsitry_logged(WOW_REG_PATH, REG_VALUE_81) {
        windows_sdk = check_add_windows_81_sdk(&path, None, windows_sdk);
    }
    if let Some(program_files_path) =
        env::var_os(PROGRAM_FILES_ENV).and_then(|p| p.into_string().ok())
    {
        windows_sdk =
            check_add_windows_81_sdk(&program_files_path, Some(WIN_KITS_PATH_81), windows_sdk);
    }
    if let Some(program_files_x86_path) =
        env::var_os(PROGRAM_FILES_X86_ENV).and_then(|p| p.into_string().ok())
    {
        windows_sdk = check_add_windows_81_sdk(
            &program_files_x86_path,
            Some(WIN_KITS_PATH_81),
            windows_sdk,
        );
    }
    windows_sdk.sort_by(|a, b| {
        a.windows_target_platform_version
            .split('.')
            .zip(b.windows_target_platform_version.split('.'))
            .map(|(j, i)| {
                let len_cmp = i.len().cmp(&j.len());
                if len_cmp != Ordering::Equal {
                    return len_cmp;
                }
                i.cmp(j)
            })
            .find(|c| c != &Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    });
    windows_sdk.dedup();
    windows_sdk
}

pub fn get_latest_windows_sdk() -> Option<String> {
    get_windows_sdk()
        .iter()
        .next()
        .map(|v| v.windows_target_platform_version.clone())
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use super::{get_windows_sdk, SdkVersion};
    use std::process::Command;

    fn get_powershell_windows_sdk_version(param_opt: Option<&str>) -> Option<String> {
        let mut args = Vec::new();
        args.push("powershell\\getWindowsSDK.ps1");
        if let Some(param) = param_opt {
            args.push(param);
        }
        let powershell_windows_sdk_output = Command::new("powershell")
            .args(&args)
            .output()
            .expect("failed to execute process");
        if (powershell_windows_sdk_output.stderr.len() > 0) {
            return None;
        }
        let powershell_windows_sdk = String::from_utf8_lossy(&powershell_windows_sdk_output.stdout);
        Some(powershell_windows_sdk.replace("\r\n", ""))
    }

    #[test]
    fn same_latest_version_than_powershell() {
        let powershell_windows_sdk = get_powershell_windows_sdk_version(None);
        assert_eq!(
            get_windows_sdk()
                .iter()
                .next()
                .map(|v| v.windows_target_platform_version.clone()),
            powershell_windows_sdk
        );
    }

    #[test]
    fn same_win10_version_than_powershell() {
        let powershell_windows_sdk = get_powershell_windows_sdk_version(Some("-DisableWin81SDK"));
        assert_eq!(
            get_windows_sdk()
                .iter()
                .filter(|v| v.windows_version == SdkVersion::Win10)
                .next()
                .map(|v| v.windows_target_platform_version.clone()),
            powershell_windows_sdk
        );
    }

    #[test]
    fn same_win8_version_than_powershell() {
        let powershell_windows_sdk = get_powershell_windows_sdk_version(Some("-DisableWin10SDK"));
        assert_eq!(
            get_windows_sdk()
                .iter()
                .filter(|v| v.windows_version == SdkVersion::Win8)
                .next()
                .map(|v| v.windows_target_platform_version.clone()),
            powershell_windows_sdk
        );
    }
}
